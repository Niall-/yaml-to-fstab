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

    let file = File::open("example.yml")?;
    let reader = BufReader::new(file);

    let m: Input = serde_yaml::from_reader(reader)?;

    let mut fstab_entries = Vec::<String>::new();
    for (device, args) in m.fstab {
        let mut fstab = String::new();

        #[allow(unused_assignments)]
        let mut device_path = String::new();

        // for now, let's just assume the yaml is correct and panic if not
        //
        // also: we're making an assumption here in that args.export is the
        // remote path on the nfs mount and args.mount is the mount point, if this
        // is wrong simply replace args.export and args.mount
        match args.fs_type.to_lowercase().as_ref() {
            "nfs" => device_path.push_str(&format!("{}:{}", device, args.export.unwrap())),
            _ => device_path.push_str(&device),
        };

        // TODO: if args.mount is a 'valid' UUID, maybe check that it's prefaced
        // with UUID=?
        fstab.push_str(&device_path);
        fstab.push_str(&format!(" {} {}", args.mount, args.fs_type));

        // again we're making an assumption here, defaulting to set the
        // fs mounting options to 'defaults' is almost certainly wrong however
        // something needs to be here and with the example.yml provided not
        // every path gives us mounting options, so: TODO: either require
        // a user provided value here (perhaps from args) and default to
        // defaults if no value is provided, or continue to always include defaults
        let mut options = match args.options.len() {
            0 => " defaults".to_string(),
            _ => format!(" defaults,{}", args.options.join(",")),
        };

        // yet again assumptions are being made as I'm really struggling
        // to figure out how to interpret "root-reserve: 10%"
        //
        // I know that ext filesystems allow you to reserve some % of the space
        // however this is usually done while creating the file system, i.e.,
        // https://man.archlinux.org/man/mke2fs.8
        // as far as mounting options goes this could mean (usr)quota but probably not
        //
        // so the assumption is this, after some googling I came across this:
        // https://fai-project.org/doc/man/setup-storage.html
        // given what FAI is and does this seems like this is the right idea
        // but could also be a red herring, however it'll allow us to pass
        // mke2fs args as fstab mounting options, where -m appears to be
        // `-m reserved-blocks-percentage`
        //
        // also TODO: since it's root-reserve maybe add a check here to see
        // if the mount point is actually root, for now let's just assume
        // that the yaml is correct
        //
        // also TODO: this will silently error if unable to parse, fix this
        match args.root_reserve {
            Some(r) => match args.fs_type.to_lowercase().as_ref() {
                "ext2" | "ext3" | "ext4" => {
                    let reserve = r.strip_suffix("%");
                    match reserve {
                        Some(r) => match r.parse::<i64>() {
                            Ok(v) => options.push_str(&format!(r#",createopts="-m {}""#, v)),
                            Err(_) => (),
                        },
                        None => (),
                    }
                }
                _ => (),
            },
            None => (),
        }

        fstab.push_str(&options);

        // finally more assumptions here about dump and pass
        // for now we'll just default to 0 0, this will be
        // easy enough to change, we can probably make some
        // safe assumptions about pass based on mount point
        // or we could just demand user input for them
        fstab.push_str(&format!(" 0 0"));

        // final sanity check
        // TODO: add some proper tests
        match device {
            m if m == "192.168.4.5" => assert_eq!(
                "192.168.4.5:/var/nfs/home /home nfs defaults,noexec,nosuid 0 0",
                fstab
            ),
            m if m == "/dev/sdb1" => assert_eq!(
                r#"/dev/sdb1 /var/lib/postgresql ext4 defaults,createopts="-m 10" 0 0"#,
                fstab
            ),
            m if m == "/dev/sda2" => assert_eq!("/dev/sda2 / ext4 defaults 0 0", fstab),
            m if m == "/dev/sda1" => assert_eq!("/dev/sda1 /boot xfs defaults 0 0", fstab),
            _ => (),
        }

        fstab_entries.push(fstab);
    }

    for f in fstab_entries {
        println!("{}", f);
    }

    Ok(())
}
