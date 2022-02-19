use std::{
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
};

use eyre::Context;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while},
    character::complete::newline,
    combinator::{eof, map_res},
    multi::many_till,
    IResult,
};
use serde::Serialize;

use super::inifile::ToIniFile;

fn version_number(input: &str) -> IResult<&str, u64> {
    map_res(take_while(char::is_numeric), |s: &str| s.parse::<u64>())(input)
}

/// Parses a version number specification.
///
/// e.g. `1000 Version`
fn version(input: &str) -> IResult<&str, u64> {
    let (input, version_number) = version_number(input)?;
    let (input, _) = tag(" Version")(input)?;
    Ok((input, version_number))
}

/// A scenery pack in the x-plane.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct SceneryPack {
    /// Whether or not this scenery pack is enabled.
    pub enabled: bool,
    /// Path to this scenery pack relative to the x-plane root
    /// directory.
    pub path: PathBuf,
}

impl ToIniFile for SceneryPack {
    type Error = eyre::Error;

    fn write_ini(&self, out: &mut impl std::io::Write) -> Result<(), Self::Error> {
        match self.enabled {
            true => {
                out.write("SCENERY_PACK".as_bytes())?;
            }
            false => {
                out.write("SCENERY_PACK_DISABLED".as_bytes())?;
            }
        }

        out.write(" ".as_bytes())?;
        out.write(self.path.as_os_str().as_bytes())
            .wrap_err_with(|| eyre::eyre!("Unable to write scenery pack path: {:?}", self.path))?;

        Ok(())
    }
}

fn scenery_pack_enabled(input: &str) -> IResult<&str, bool> {
    let enabled_tag = map_res(tag("SCENERY_PACK"), |_| Result::<bool, ()>::Ok(true));
    let disabled_tag = map_res(tag("SCENERY_PACK_DISABLED"), |_| {
        Result::<bool, ()>::Ok(false)
    });

    alt((disabled_tag, enabled_tag))(input)
}

fn until_newline(input: &str) -> IResult<&str, &Path> {
    let (input, path_str) = is_not("\r\n")(input)?;
    let path = Path::new(path_str);

    Ok((input, path))
}

/// Parse a scenery pack line in the ini file.
///
/// e.g. `SCENERY_PACK Custom Scenery/Some Pack` or
/// `SCENERY_PACK_DISABLED Custom Scenery/Some Pack`.
fn scenery_pack(input: &str) -> IResult<&str, SceneryPack> {
    let (input, enabled) = scenery_pack_enabled(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, path) = until_newline(input)?;

    let output = SceneryPack {
        enabled,
        path: path.to_owned(),
    };

    Ok((input, output))
}

fn scenery_pack_or_newline(input: &str) -> IResult<&str, Option<SceneryPack>> {
    alt((
        map_res(newline, |_| Result::<Option<SceneryPack>, ()>::Ok(None)),
        map_res(scenery_pack, |pack| {
            Result::<Option<SceneryPack>, ()>::Ok(Some(pack))
        }),
    ))(input)
}

#[derive(Serialize, Clone, Debug)]
pub struct SceneryPacksIni {
    pub version: u64,
    pub scenery_packs: Vec<SceneryPack>,
}

impl ToIniFile for SceneryPacksIni {
    type Error = eyre::Error;

    fn write_ini(&self, out: &mut impl std::io::Write) -> Result<(), Self::Error> {
        out.write("I\n".as_bytes())?;

        out.write(self.version.to_string().as_bytes())?;
        out.write(" Version\n".as_bytes())?;

        out.write("SCENERY\n\n".as_bytes())?;

        for pack in &self.scenery_packs {
            pack.write_ini(out)?;
            out.write("\n".as_bytes())?;
        }

        Ok(())
    }
}

pub fn scenery_packs_ini(input: &str) -> IResult<&str, SceneryPacksIni> {
    let (input, _) = tag("I")(input)?;
    let (input, _) = newline(input)?;
    let (input, version_number) = version(input)?;
    let (input, _) = newline(input)?;
    let (input, _) = tag("SCENERY")(input)?;
    let (input, _) = newline(input)?;

    let (input, (lines, _)) = many_till(scenery_pack_or_newline, eof)(input)?;
    let scenery_packs: Vec<SceneryPack> = lines.into_iter().filter_map(|line| line).collect();

    let output = SceneryPacksIni {
        version: version_number,
        scenery_packs,
    };
    Ok((input, output))
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::{
        scenery_pack, scenery_pack_enabled, scenery_packs_ini, until_newline, version, SceneryPack,
        SceneryPacksIni, ToIniFile,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_version() {
        assert_eq!(1000, version("1000 Version").unwrap().1);
        assert!(version("1000").is_err());
    }

    #[test]
    fn test_scenery_packs_ini() {
        let ini_file = std::fs::read_to_string("src/parsers/scenery_packs.ini").unwrap();
        insta::assert_json_snapshot!(scenery_packs_ini(&ini_file).unwrap().1);
    }

    #[test]
    fn test_scenery_pack_enabled() {
        assert!(scenery_pack_enabled("SCENERY_PACK").unwrap().1);
        assert!(!scenery_pack_enabled("SCENERY_PACK_DISABLED").unwrap().1);
    }

    #[test]
    fn test_path_until_newline() {
        assert_eq!(
            Path::new("Custom Scenery/-Canberra_Beacons/"),
            until_newline("Custom Scenery/-Canberra_Beacons/\n")
                .unwrap()
                .1
        );
    }

    #[test]
    fn test_scenery_pack() {
        let enabled_pack = scenery_pack("SCENERY_PACK Custom Scenery/-Canberra_Beacons/\n")
            .unwrap()
            .1;
        insta::assert_json_snapshot!(enabled_pack);
        let disabled_pack =
            scenery_pack("SCENERY_PACK_DISABLED Custom Scenery/A New Zealand NZMC Mount Cook/\n")
                .unwrap()
                .1;
        insta::assert_json_snapshot!(disabled_pack);
    }

    #[test]
    fn test_to_ini() {
        let pack1 = SceneryPack {
            enabled: true,
            path: "Example 1".into(),
        };

        let pack2 = SceneryPack {
            enabled: false,
            path: "Example 2".into(),
        };

        let scenery_packs_ini = SceneryPacksIni {
            version: 1000,
            scenery_packs: vec![pack1, pack2],
        };

        let mut buffer: Vec<u8> = Vec::new();

        scenery_packs_ini.write_ini(&mut buffer).unwrap();

        let ini_file = std::str::from_utf8(&buffer).unwrap();

        let expected = r#"I
1000 Version
SCENERY

SCENERY_PACK Example 1
SCENERY_PACK_DISABLED Example 2
"#;
        assert_eq!(expected, ini_file)
    }
}
