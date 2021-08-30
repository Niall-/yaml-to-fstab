use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "yaml-to-fstab")]
struct Opt {
    #[structopt(short, long)]
    conf: String,
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
    //let opt = Opt::from_args();
    //let file = File::open(&opt.conf)?;

    let file = File::open("example.yml")?;
    let reader = BufReader::new(file);

    let mounts: Input = serde_yaml::from_reader(reader)?;

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
            "nfs" => fs_spec.push_str(&format!("{}:{}", &device, args.export.unwrap())),
            _ => fs_spec.push_str(&device),
        };

        // TODO: if args.mount is a 'valid' UUID, maybe check that it's prefaced
        // with UUID=?
        entry.push_str(&fs_spec);
        entry.push_str(&format!(" {} {}", args.mount, args.fs_type));

        // always including 'defaults' in mntops is most certainly the wrong thing
        // to do here, however as the example yaml file doesn't provide a default
        // value to use we're left to guess what should be here
        //
        // alternatives for what to do include accepting user input while running the tool
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
        //
        // also TODO: since it's root-reserve maybe add a check here to see
        // if the mount point is actually root, for now let's just assume
        // that the yaml is correct
        match args.root_reserve {
            Some(r) => match args.fs_type.to_lowercase().as_ref() {
                "ext2" | "ext3" | "ext4" => {
                    // this parsing could go very wrong, assume that it's consistent and panic if not
                    let reserve = r.strip_suffix("%").unwrap().parse::<i64>().unwrap();
                    fs_mntops.push_str(&format!(r#",createopts="-m {}""#, reserve));
                    //match reserve {
                    //    Some(r) => match r.parse::<i64>() {
                    //        Ok(v) => options.push_str(&format!(r#",createopts="-m {}""#, v)),
                    //        Err(_) => (),
                    //    },
                    //    None => (),
                    //}
                }
                // probably worth panicking here as well
                _ => panic!("root-reserve was provided on a non-ext filesystem"),
            },
            None => (),
        }

        entry.push_str(&fs_mntops);

        // finally more assumptions here about dump and pass
        // for now we'll just default to 0 0, this will be
        // easy enough to change, we can probably make some
        // safe assumptions about pass based on mount point
        // or we could just demand user input for them like
        // with fs_mntops
        entry.push_str(&format!(" 0 0"));

        //// finally some sanity checks with hand written entries
        //// TODO: add some proper tests
        //match device {
        //    m if m == "192.168.4.5" => assert_eq!(
        //        "192.168.4.5:/var/nfs/home /home nfs defaults,noexec,nosuid 0 0",
        //        entry
        //    ),
        //    m if m == "/dev/sdb1" => assert_eq!(
        //        r#"/dev/sdb1 /var/lib/postgresql ext4 defaults,createopts="-m 10" 0 0"#,
        //        entry
        //    ),
        //    m if m == "/dev/sda2" => assert_eq!("/dev/sda2 / ext4 defaults 0 0", entry),
        //    m if m == "/dev/sda1" => assert_eq!("/dev/sda1 /boot xfs defaults 0 0", entry),
        //    _ => (),
        //}

        fstab_entries.push(entry);
    }

    for f in fstab_entries {
        println!("{}", f);
    }

    Ok(())
}
