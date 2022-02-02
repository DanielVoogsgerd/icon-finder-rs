use std::path::Path;
use std::path::PathBuf;
pub fn new(theme: Theme) -> IconFinderInstance {
    IconFinderInstance {theme}
}

pub struct IconFinderInstance {
    pub theme: Theme
}

impl IconFinderInstance {
    pub fn find_icon(self, icon: &str, size: i16, scale: i16) -> Option<String> {
        find_icon(icon, size, scale, self.theme)
    }
}

pub struct Icon {
    pub theme: Theme
}


pub struct UnloadedTheme {
    location: Path
}

impl UnloadedTheme {
    fn load(self) -> Theme {
        Theme {
            name: String::from("Insert name of theme"),
            comment: String::from("Insert comment after it's read"),
            inherits: vec!(),
            location: self.location,
            directories: vec!()

        }
    }
}

/// Find the fallback Hicolor theme
fn find_fallback_theme() -> UnloadedTheme {
    for directory in &BASE_DIRECTORIES {
        let path = Path::new(&format!("{}/index.theme", directory));
        if path.exists() {
            return UnloadedTheme {
                location: path
            }
        }
    }
}

/// Icon Theme Specification
/// ========================
/// Find icons for applications according to the freedesktop.org specifications

pub fn get_user_selected_theme() -> String {
    // TODO: Actually fetch the theme
    return "/usr/share/themes/Adwaita/index.theme".to_string();
}

/// # Icon Theme
/// An icon theme is a named set of icons. It is used to map from an iconname
/// and size to a file. Themes may inherit from other themes as a way to extend
/// them.
// TODO: Figure out what location takes presidence in case of multiple locations
pub struct Theme {
    pub name: String,
    pub comment: String,
    pub inherits: Vec<Theme>,
    pub directories: Vec<ThemeDirectory>,
    pub location: Path,
}

/// # Per directory keys
/// Each directory specified in the Directory key has a corresponding section
/// with the same name as the directory. The contents of this section is listed
/// in table 2 below.
pub struct ThemeDirectory {
    pub name: String,
    pub size: i16,
    pub scale: Option<i16>,
    pub context: Option<String>,
    pub r#type: ThemeDirectoryType,
    pub max_size: Option<i16>,
    pub min_size: Option<i16>,
    pub threshold: Option<i16>,
}

/// # Per directory key types
/// The type of icon sizes for the icons in this directory. Valid types are
/// Fixed, Scalable and Threshold. The type decides what other keys in the
/// section are used. If not specified, the default is Threshold.
/// TODO: Define Threshold as default
pub enum ThemeDirectoryType {
    Fixed,
    Scalable,
    Threshold,
}

// The fallback theme in this case is the hicolor theme, as mentioned in the specification.

/// # Base directories
/// Icons and themes are searched for in a set of directories, called base
/// directories. The themes are stored in subdirectories of the base
/// directories.
const BASE_DIRECTORIES: [&str; 3] = ["~/.icons", "/usr/share/icons", "/usr/local/share/icons"];

/// An icon file is an image that can be loaded and used as an icon. The
/// supported image file formats are PNG, XPM and SVG. PNG is the recommended
/// bitmap format, and SVG is for vectorized icons. XPM is supported due to
/// backwards compability reasons, and it is not recommended that new themes use
/// XPM files. Support for SVGs is optional.

// TODO: Make svg/xpm optional
const ALLOWED_EXTENSIONS: [&str; 3] = ["png", "svg", "xpm"];
const DEFAULT_THRESHOLD: i16 = 2;
const DEFAULT_SCALE: i16 = 1;

/// # Icon Lookup
/// From: https://standards.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html#icon_lookup
/// The icon lookup mechanism has two global settings, the list of base
/// directories and the internal name of the current theme. Given these we need
/// to specify how to look up an icon file from the icon name, the nominal size
/// and the scale.
/// The lookup is done first in the current theme, and then recursively in each
/// of the current theme's parents, and finally in the default theme called
/// "hicolor" (implementations may add more default themes before "hicolor",
/// but "hicolor" must be last). As soon as there is an icon of any size that
/// matches in a theme, the search is stopped. Even if there may be an icon with
/// a size closer to the correct one in an inherited theme, we don't want to use
/// it. Doing so may generate an inconsistant change in an icon when you change
/// icon sizes (e.g. zoom in).
/// The lookup inside a theme is done in three phases. First all the directories
/// are scanned for an exact match, e.g. one where the allowed size of the icon
/// files match what was looked up. Then all the directories are scanned for any
/// icon that matches the name. If that fails we finally fall back on unthemed
/// icons. If we fail to find any icon at all it is up to the application to
/// pick a good fallback, as the correct choice depends on the context.
pub fn find_icon(icon: &str, size: i16, scale: i16, user_selected_theme: Theme) -> Option<String> {
    // TODO: Flatten this function
    let fallback_theme: Theme = Theme {
        name: "hicolor".to_owned(),
        comment: "Default icon theme".to_owned(),
        inherits: Vec::new(),
        directories: vec![],
    };

    return match find_icon_helper(icon, size, scale, &user_selected_theme) {
        Some(icon) => Some(icon),
        None => {
            return match find_icon_helper(icon, size, scale, &fallback_theme) {
                Some(icon) => Some(icon),
                None => return None,
            };
        }
    };
}

/// In some cases you don't always want to fall back to an icon in an inherited
/// theme. For instance, sometimes you look for a set of icons, prefering any of
/// them before using an icon from an inherited theme. To support such
/// operations implementations can contain a function that finds the first of a
/// list of icon names in the inheritance hierarchy. I.E. It would look
/// something like this:
pub fn find_best_icon(
    // TODO: Flatten this function
    icon_list: Vec<&str>,
    size: i16,
    scale: i16,
    user_selected_theme: Theme,
) -> Option<String> {
    let fallback_theme: Theme = Theme {
        name: "hicolor".to_owned(),
        comment: "Default icon".to_owned(),
        inherits: Vec::new(),
        directories: vec![],
    };

    return match find_best_icon_helper(&icon_list, size, scale, &user_selected_theme) {
        Some(filename) => Some(filename),
        None => {
            return match find_best_icon_helper(&icon_list, size, scale, &fallback_theme) {
                Some(filename) => Some(filename),
                None => {
                    for icon in icon_list {
                        let filename = match lookup_fallback_icon(icon) {
                            Some(filename) => filename,
                            None => {
                                continue;
                            }
                        };
                        return Some(filename);
                    }

                    return None;
                }
            };
        }
    };
}

/// # Implementation Notes
/// The algorithm as described in this document works by always looking up
/// filenames in directories (a stat in unix terminology). A good implementation
/// is expected to read the directories once, and do all lookups in memory using
/// that information.
/// This caching can make it impossible for users to add icons without having to
/// restart applications. In order to handle this, any implementation that does
/// caching is required to look at the mtime of the toplevel icon directories
/// when doing a cache lookup, unless it already did so less than 5 seconds ago.
/// This means that any icon editor or theme installation program need only to
/// change the mtime of the the toplevel directory where it changed the theme to
/// make sure that the new icons will eventually get used.
fn find_icon_helper(icon: &str, size: i16, scale: i16, theme: &Theme) -> Option<String> {
    // TODO: Flatten this function
    let filename = match lookup_icon(icon, size, scale, theme) {
        Some(f) => Some(f),
        None => {
            // The check from the pseudocode can be left out because we force parents to be set.
            for parent in &theme.inherits {
                match find_icon_helper(icon, size, scale, &parent) {
                    Some(f) => return Some(f),
                    None => continue,
                }
            }

            return None;
        }
    };

    return filename;
}

fn find_best_icon_helper(
    icon_list: &Vec<&str>,
    size: i16,
    scale: i16,
    theme: &Theme,
) -> Option<String> {
    // TODO: Flatten this function
    for icon in icon_list {
        let filename = match lookup_icon(icon, size, scale, theme) {
            Some(f) => f,
            None => continue,
        };

        return Some(filename);
    }

    for parent in &theme.inherits {
        let filename = match find_best_icon_helper(icon_list, size, scale, &parent) {
            Some(f) => f,
            None => {
                continue;
            }
        };

        return Some(filename);
    }

    return None;
}

fn lookup_icon(icon_name: &str, size: i16, scale: i16, theme: &Theme) -> Option<String> {
    for subdir in &theme.directories {
        for directory in &BASE_DIRECTORIES {
            for extension in &ALLOWED_EXTENSIONS {
                if directory_matches_size(subdir, size, scale) {
                    let file_path = format!(
                        "{directory}/{theme_name}/{subdir}/{icon_name}.{extension}",
                        directory = directory,
                        theme_name = theme.name,
                        subdir = subdir.name,
                        icon_name = icon_name,
                        extension = extension
                    );

                    if Path::new(&file_path).exists() {
                        return Some(file_path);
                    }
                }
            }
        }
    }

    // No exact match was found, compute the closest matching icon.
    // TODO: There is a more elegant solution than this
    let mut minimal_size = i16::max_value();
    let mut closest_filename = String::from("");

    for subdir in &theme.directories {
        for directory in &BASE_DIRECTORIES {
            for extension in &ALLOWED_EXTENSIONS {
                let file_path = format!(
                    "{directory}/{theme_name}/{subdir}/{icon_name}.{extension}",
                    directory = directory,
                    theme_name = theme.name,
                    subdir = subdir.name,
                    icon_name = icon_name,
                    extension = extension
                );

                let directory_size_distance = directory_size_distance(&subdir, size, scale);
                if Path::new(&file_path).exists() && directory_size_distance < minimal_size {
                    // Found a better match, updating closest file
                    closest_filename = file_path;
                    minimal_size = directory_size_distance;
                }
            }
        }
    }

    if minimal_size < i16::max_value() {
        return Some(closest_filename);
    }
    return None;
}

fn lookup_fallback_icon(icon_name: &str) -> Option<String> {
    for directory in &BASE_DIRECTORIES {
        for extension in &ALLOWED_EXTENSIONS {
            let file_path = format!(
                "{directory}/{icon_name}.{extension}",
                directory = directory,
                icon_name = icon_name,
                extension = extension
            );

            if Path::new(&file_path).exists() {
                return Some(file_path);
            }
        }
    }

    return None;
}

fn directory_matches_size(theme_directory: &ThemeDirectory, icon_size: i16, icon_scale: i16) -> bool {
    if icon_scale != theme_directory.scale.unwrap_or(DEFAULT_SCALE) {
        return false;
    }

    let min_size = theme_directory.min_size.unwrap_or(theme_directory.size);
    let max_size = theme_directory.max_size.unwrap_or(theme_directory.size);
    println!("{:?}", icon_size);
    println!("{:?}", min_size);
    println!("{:?}", max_size);
    let threshold = theme_directory.threshold.unwrap_or(DEFAULT_THRESHOLD);

    return match theme_directory.r#type {
        ThemeDirectoryType::Fixed => {
            println!("Fixed");
            theme_directory.size == icon_size },
        ThemeDirectoryType::Scalable => { min_size <= icon_size && icon_size <= max_size },
        ThemeDirectoryType::Threshold => {
            theme_directory.size - threshold <= icon_size
                && icon_size <= theme_directory.size + threshold
        }
    };
}

/// Watch out with threshold! The distance is 0 as long as the icon_size * icon_scale is between
/// the threshold. But as soon as it's outside of the threshold the distance is calculated from the
/// set icon size for the directory. Which is equal to threshold.
/// This is not identical to the spec: Read more here: https://github.com/DanielVoogsgerd/icon-finder-rs/issues/3
fn directory_size_distance(theme_directory: &ThemeDirectory, icon_size: i16, icon_scale: i16) -> i16 {
    let min_size = theme_directory.min_size.unwrap_or(theme_directory.size);
    let max_size = theme_directory.max_size.unwrap_or(theme_directory.size);
    let threshold = theme_directory.threshold.unwrap_or(DEFAULT_THRESHOLD);
    let theme_directory_scale = theme_directory.scale.unwrap_or(DEFAULT_SCALE);

    return match theme_directory.r#type {
        ThemeDirectoryType::Fixed => {
            // FIXME: The integers are signed because of this line. On one hand I could split this
            // up into two lines and make them unsigned, but it might also be more hassle than that
            // it's worth.
            (theme_directory.size * theme_directory_scale - icon_size * icon_scale).abs()
        }
        ThemeDirectoryType::Scalable => {
            if icon_size * icon_scale < min_size * theme_directory_scale {
                return min_size * theme_directory_scale - icon_size * icon_scale;
            }

            if icon_size * icon_scale > max_size * theme_directory_scale {
                return icon_size * icon_scale - max_size * theme_directory_scale;
            }

            return 0;
        }
        ThemeDirectoryType::Threshold => {
            if icon_size * icon_scale < (theme_directory.size - threshold) * theme_directory_scale {
                return theme_directory.size * theme_directory_scale - icon_size * icon_scale;
            }

            if icon_size * icon_scale > (theme_directory.size + threshold) * theme_directory_scale {
                return icon_size * icon_scale - theme_directory.size * theme_directory_scale;
            }

            return 0;
        }
    };
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_directory_matches_size_different_scale() {
        let theme_directory = ThemeDirectory {
            name: "Main".to_owned(),
            size: 512,
            scale: Some(1),
            context: Some("actions".to_owned()),
            r#type: ThemeDirectoryType::Fixed,
            min_size: None,
            max_size: None,
            threshold: None,
        };

        assert_eq!(directory_matches_size(&theme_directory, 512, 2), false);
    }

    #[test]
    fn test_directory_matches_size_fixed() {
        let theme_directory = ThemeDirectory {
            name: "Main".to_owned(),
            size: 512,
            scale: Some(1),
            context: Some("actions".to_owned()),
            r#type: ThemeDirectoryType::Fixed,
            min_size: None,
            max_size: None,
            threshold: None,
        };

        assert_eq!(directory_matches_size(&theme_directory, 512, 1), true);
        assert_eq!(directory_matches_size(&theme_directory, 256, 1), false);
    }

    #[test]
    fn test_directory_matches_size_scalable() {
        let theme_directory = ThemeDirectory {
            name: "Main".to_owned(),
            size: 512,
            scale: Some(1),
            context: Some("actions".to_owned()),
            r#type: ThemeDirectoryType::Scalable,
            min_size: Some(256),
            max_size: Some(1024),
            threshold: None,
        };

        assert_eq!(directory_matches_size(&theme_directory, 128, 1), false);
        assert_eq!(directory_matches_size(&theme_directory, 256, 1), true);
        assert_eq!(directory_matches_size(&theme_directory, 511, 1), true);
        assert_eq!(directory_matches_size(&theme_directory, 512, 1), true);
        assert_eq!(directory_matches_size(&theme_directory, 1024, 1), true);
        assert_eq!(directory_matches_size(&theme_directory, 2048, 1), false);
    }

    #[test]
    fn test_directory_matches_size_threshold() {
        let theme_directory = ThemeDirectory {
            name: "Main".to_owned(),
            size: 512,
            scale: Some(1),
            context: Some("actions".to_owned()),
            r#type: ThemeDirectoryType::Threshold,
            min_size: Some(256),
            max_size: Some(1024),
            threshold: Some(128),
        };

        assert_eq!(directory_matches_size(&theme_directory, 128, 1), false);
        assert_eq!(directory_matches_size(&theme_directory, 384, 1), true);
        assert_eq!(directory_matches_size(&theme_directory, 512, 1), true);
        assert_eq!(directory_matches_size(&theme_directory, 640, 1), true);
        assert_eq!(directory_matches_size(&theme_directory, 1025, 1), false);
    }

    // Tests for directory_size_difference
    #[test]
    fn test_directory_size_distance_fixed() {
        let theme_directory = ThemeDirectory {
            name: "Main".to_owned(),
            size: 512,
            scale: Some(1),
            context: Some("actions".to_owned()),
            r#type: ThemeDirectoryType::Fixed,
            min_size: Some(256),
            max_size: Some(1024),
            threshold: Some(128),
        };

        assert_eq!(directory_size_distance(&theme_directory, 512, 1), 0);
        assert_eq!(directory_size_distance(&theme_directory, 256, 2), 0);
        assert_eq!(directory_size_distance(&theme_directory, 100, 1), 412);
        assert_eq!(directory_size_distance(&theme_directory, 1512, 1), 1000);

    }

    #[test]
    fn test_directory_size_distance_scalable() {
        let theme_directory = ThemeDirectory {
            name: "Main".to_owned(),
            size: 512,
            scale: Some(1),
            context: Some("actions".to_owned()),
            r#type: ThemeDirectoryType::Scalable,
            min_size: Some(256),
            max_size: Some(1024),
            threshold: Some(128),
        };

        assert_eq!(directory_size_distance(&theme_directory, 128, 1), 128);
        assert_eq!(directory_size_distance(&theme_directory, 64, 2), 128);
        assert_eq!(directory_size_distance(&theme_directory, 256, 1), 0);
        assert_eq!(directory_size_distance(&theme_directory, 512, 1), 0);
        assert_eq!(directory_size_distance(&theme_directory, 1024, 1), 0);
        assert_eq!(directory_size_distance(&theme_directory, 2024, 1), 1000);
    }

    #[test]
    fn test_directory_size_distance_threshold() {
        let theme_directory = ThemeDirectory {
            name: "Main".to_owned(),
            size: 512,
            scale: Some(1),
            context: Some("actions".to_owned()),
            r#type: ThemeDirectoryType::Threshold,
            min_size: Some(256),
            max_size: Some(1024),
            threshold: Some(128),
        };

        assert_eq!(directory_size_distance(&theme_directory, 256, 1), 256);
        assert_eq!(directory_size_distance(&theme_directory, 384, 1), 0);
        assert_eq!(directory_size_distance(&theme_directory, 512, 1), 0);
        assert_eq!(directory_size_distance(&theme_directory, 640, 1), 0);
        assert_eq!(directory_size_distance(&theme_directory, 768, 1), 256);
    }
}
