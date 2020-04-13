use globset::{Error, Glob, GlobSet, GlobSetBuilder};

use crate::configuration::{MatcherConfig, MatcherSpec};

pub struct Matcher {
    build_result: GlobSet,
    skip_globset: GlobSet,
}

impl Matcher {
    pub fn new(matcher_config: &Option<MatcherConfig>) -> Result<Self, String> {
        let mut ignore_spec_builder = GlobSetBuilder::new();
        let mut skip_spec_builder = GlobSetBuilder::new();

        if let Some(mc) = matcher_config {
            if let Some(ignore_specs) = &mc.ignore_specs {
                let build_result = build_globset(&mut ignore_spec_builder, &ignore_specs);
                if build_result.is_err() {
                    return Err(build_result.err().unwrap().to_string());
                }
            }

            if let Some(skip_specs) = &mc.skip_specs {
                let build_result = build_globset(&mut skip_spec_builder, &skip_specs);
                if build_result.is_err() {
                    return Err(build_result.err().unwrap().to_string());
                }
            }
        }

        let matcher = Self {
            build_result: ignore_spec_builder.build().unwrap(),
            skip_globset: skip_spec_builder.build().unwrap(),
        };

        Ok(matcher)
    }

    pub fn matches(&self, path: String) -> bool {
        let ignore_match = self.build_result.is_match(&path);

        if !ignore_match {
            return false;
        }

        let skip_match = self.skip_globset.is_match(&path);

        !skip_match
    }
}

fn build_globset(
    builder: &mut GlobSetBuilder,
    matcher_specs: &Vec<MatcherSpec>,
) -> Result<(), Error> {
    for matcher_spec in matcher_specs.iter() {
        let glob = Glob::new(&matcher_spec.pattern);

        if glob.is_err() {
            return Err(glob.err().unwrap());
        }

        builder.add(glob.unwrap());
    }
    builder.build()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest(
        path,
        expected,
        case::on_root("/build", false),
        case::on_sub("/foo/build", false),
        case::on_sub_sub("/foo/bar/build", false),
        case::on_other_spec("/foo/target", false),
        case::not_matching_in_between("/foo/build/bar", false),
        case::not_matching_other("/foo/bar", false),
        case::not_matching_specific("/foo/src/build", false)
    )]
    fn no_matches_with_empty_config(path: &str, expected: bool) {
        let matcher_config = MatcherConfig {
            ignore_specs: None,
            skip_specs: None,
        };

        let matcher = Matcher::new(&Some(matcher_config));

        assert_eq!(expected, matcher.unwrap().matches(path.to_string()));
    }

    #[rstest(
        path,
        expected,
        case::on_root("/build", true),
        case::on_sub("/foo/build", true),
        case::on_sub_sub("/foo/bar/build", true),
        case::on_other_spec("/foo/target", true),
        case::not_matching_in_between("/foo/build/bar", false),
        case::not_matching_other("/foo/bar", false),
        case::not_matching_specific("/foo/src/build", false)
    )]
    fn matches_with_config(path: &str, expected: bool) {
        let matcher_config = MatcherConfig {
            ignore_specs: Some(vec![
                MatcherSpec {
                    pattern: String::from("**/build"),
                },
                MatcherSpec {
                    pattern: String::from("**/target"),
                },
            ]),
            skip_specs: Some(vec![MatcherSpec {
                pattern: String::from("**/src/build"),
            }]),
        };

        let matcher = Matcher::new(&Some(matcher_config));

        assert_eq!(expected, matcher.unwrap().matches(path.to_string()));
    }

    #[rstest(
        matcher_config,
        case::ignore_specs(MatcherConfig {
            ignore_specs: Some(vec![
                MatcherSpec {
                    pattern: String::from("**/src/bu{ild"),
                }
            ]),
            skip_specs: None,
        }),
        case::skip_specs(MatcherConfig {
            ignore_specs: None,
            skip_specs: Some(vec![
                MatcherSpec {
                    pattern: String::from("**/src/bu{ild"),
                }
            ]),
        }),
    )]
    fn broken_configuration(matcher_config: MatcherConfig) {
        let matcher = Matcher::new(&Some(matcher_config));

        assert!(matcher.is_err());
        assert!(matcher.err().unwrap().contains("error parsing glob"))
    }
}
