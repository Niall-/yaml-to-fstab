use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "yaml-to-fstab")]
struct Opt {
    #[structopt(short, long, help = "Input yaml to parse")]
    input: String,
    #[structopt(short, long, help = "Performs a dry run")]
    dry_run: bool,
    #[structopt(
        long,
        default_value = "0",
        help = "Global value for fs_freq/dump\n\
                Should be either 0 or 1"
    )]
    dump: isize,
    #[structopt(
        long,
        default_value = "0",
        // I don't think this is true but any other value seems unusual
        help = "Global value for fs_passno/fsck\n\
                Should be either 0, 1, or 2"
    )]
    fsck: isize,
    // TODO: this could probably be more granular with /boot and remote filesytems
    #[structopt(
        long,
        help = "Sets the root partition to 1, all other partitions to 2\n\
                If enabled, ignores --dump and --fsck options"
    )]
    smart_fsck: bool,
}

#[derive(Deserialize, Debug)]
struct Input {
    fstab: HashMap<String, Mounts>,
}

#[derive(Deserialize, Debug)]
struct Mounts {
    mount: String,
    export: Option<String>,
    #[serde(rename = "type")]
    fs_type: String,
    #[serde(rename = "root-reserve")]
    root_reserve: Option<String>,
    #[serde(default)]
    options: Vec<String>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    match opt.dump {
        0 | 1 => (),
        _ => panic!("--dump should be either 0 or 1"),
    }
    match opt.fsck {
        0 | 1 | 2 => (),
        _ => panic!("--fsck should be between 0 and 2"),
    }

    let input_path = Path::new(&opt.input);
    let input_file = File::open(input_path).expect("Unable to open input yaml file");
    let input_reader = BufReader::new(input_file);

    let fstab_path = Path::new("/etc/fstab");
    let fstab_file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(fstab_path);

    if !opt.dry_run {
        if let Err(e) = fstab_file {
            panic!("Unable to open /etc/fstab: {}", e);
        }
    }

    let mounts: Input = serde_yaml::from_reader(input_reader)?;

    let mut fstab_entries = Vec::<String>::new();
    for (device, args) in mounts.fstab {
        let mut entry = String::new();

        #[allow(unused_assignments)]
        let mut fs_spec = String::new();
        // args.export may be empty, for now let's just assume the yaml is correct
        // and provides the value or panic if not
        //
        // NOTE: we're making an assumption here in that args.export is the
        // remote path on the nfs mount and args.mount is the mount point, if this
        // is wrong simply replace args.export and args.mount
        match args.fs_type.to_lowercase().as_ref() {
            "nfs" => {
                let export = args.export.as_ref().expect("NFS mount with no export path");
                fs_spec.push_str(&format!("{}:{}", &device, export));
            }
            _ => fs_spec.push_str(&device),
        };
        entry.push_str(&fs_spec);
        entry.push_str(&format!(" {} {}", args.mount, args.fs_type));

        // always including 'defaults' in mntops is most certainly the wrong thing
        // to do here, however as the example yaml file doesn't provide a default
        // value to use we're left to guess what should be here
        let mut fs_mntops = match args.options.len() {
            0 => " defaults".to_string(),
            _ => format!(" defaults,{}", args.options.join(",")),
        };

        // yet again assumptions are being made as I'm really struggling
        // to figure out how to interpret "root-reserve: 10%"
        //
        // I know that ext filesystems allow you to reserve some % of the space
        // however this is usually done while creating the filesystem, i.e.,
        // https://man.archlinux.org/man/mke2fs.8
        // as far as mounting options goes this could mean (usr)quota but probably not
        //
        // so the assumption is this, after some googling I came across this:
        // https://fai-project.org/doc/man/setup-storage.html
        // given what FAI is and does this seems like it's the right idea
        // but could also be a red herring, however it'll allow us to pass
        // mke2fs args as fstab mounting options, where -m appears to be
        // `-m reserved-blocks-percentage`
        if let Some(r) = args.root_reserve.as_ref() {
            match args.fs_type.to_lowercase().as_ref() {
                "ext2" | "ext3" | "ext4" => {
                    // this parsing could go very wrong, assume that it's consistent and panic if not
                    let reserve = r
                        .strip_suffix("%")
                        .expect("Unable to parse value for root-reserve")
                        .parse::<i64>()
                        .expect("Unable to parse value for root-reserve");
                    fs_mntops.push_str(&format!(r#",createopts="-m {}""#, reserve));
                }
                // probably worth panicking here as well
                _ => panic!("root-reserve was provided on a non-ext filesystem"),
            }
        }
        entry.push_str(&fs_mntops);

        match opt.smart_fsck {
            true => {
                let root = match args.mount.as_ref() {
                    "/" => 1,
                    _ => 2,
                };
                entry.push_str(&format!(" {} {}", opt.dump, root));
            }
            false => entry.push_str(&format!(" {} {}", opt.dump, opt.fsck)),
        }

        // tests shouldn't be necessary as serde will panic
        // if the yaml doesn't contain a mount point and fs type
        // and those are things we can't really make assumptions about

        // TODO: we can probably align all the entries with Rust's formatter
        fstab_entries.push(entry);
    }

    match opt.dry_run {
        false => {
            // this is checked at the start of the file but still
            // needs to be unwrapped
            let mut writer = BufWriter::new(fstab_file.expect("Unable to open /etc/fstab"));
            for f in fstab_entries {
                let entry = format!("{}\n", f);
                writer.write(entry.as_bytes())?;
                println!("Adding to /etc/fstab:    {}", f);
            }
            writer.flush()?;
        }
        true => {
            println!("--- dry run ---");
            for f in fstab_entries {
                println!("Adding to /etc/fstab:    {}", f);
            }
            println!("--- dry run ---");
        }
    }

    Ok(())
}
