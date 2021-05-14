use std::{
    path::{Path, PathBuf},
};

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while},
    character::complete::newline,
    combinator::{eof, map_res},
    multi::many_till,
    IResult,
};
use serde::Serialize;

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

#[derive(Serialize)]
pub struct SceneryPack {
    pub enabled: bool,
    pub path: PathBuf,
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

#[derive(Serialize)]
pub struct SceneryPacksIni {
    pub version: u64,
    pub scenery_packs: Vec<SceneryPack>,
}

fn scenery_packs_ini(input: &str) -> IResult<&str, SceneryPacksIni> {
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

    use super::{scenery_pack, scenery_pack_enabled, scenery_packs_ini, until_newline, version};
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
}
