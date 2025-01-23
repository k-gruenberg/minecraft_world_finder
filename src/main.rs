use std::{env, fs};
use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use phf::{phf_map};
use std::fs::File;
use std::io::BufReader;
use flate2::read::GzDecoder;
use std::io::Read;
use std::sync::Mutex;
use chrono::prelude::DateTime;
use chrono::Utc;
use std::time::{UNIX_EPOCH, Duration};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{thread, time};

/// An integer indicating the Minecraft version.
/// Cf. https://minecraft.wiki/w/Data_version
/// Introduced with version 1.9	(15w32a).
/// Cf. https://minecraft.wiki/w/Java_Edition_version_history#Full_release for release dates.
static DATA_VERSIONS: phf::Map<i32, &'static str> = phf_map! {
    // Version 1.21:
    4189i32 => "1.21.4 - December 3, 2024",
    4082i32 => "1.21.3 - October 23, 2024",
    4080i32 => "1.21.2 - October 22, 2024",
    3955i32 => "1.21.1 - August 8, 2024",
    3953i32 => "1.21 - June 13, 2024",

    // Version 1.20:
    3839i32 => "1.20.6 - April 29, 2024",
    3837i32 => "1.20.5 - April 23, 2024",
    3700i32 => "1.20.4 - December 7, 2023",
    3698i32 => "1.20.3 - December 5, 2023",
    3578i32 => "1.20.2 - September 21, 2023",
    3465i32 => "1.20.1 - June 12, 2023",
    3463i32 => "1.20 - June 7, 2023",

    // Version 1.19:
    3337i32 => "1.19.4 - March 14, 2023",
    3218i32 => "1.19.3 - December 7, 2022",
    3120i32 => "1.19.2 - August 5, 2022",
    3117i32 => "1.19.1 - July 27, 2022",
    3105i32 => "1.19 - June 7, 2022",

    // Version 1.18:
    2975i32 => "1.18.2 - February 28, 2022",
    2865i32 => "1.18.1 - December 10, 2021",
    2860i32 => "1.18 - November 30, 2021",

    // Version 1.17:
    2730i32 => "1.17.1 - July 6, 2021",
    2724i32 => "1.17 - June 8, 2021",

    // Version 1.16:
    2586i32 => "1.16.5 - January 15, 2021",
    2584i32 => "1.16.4 - November 2, 2020",
    2580i32 => "1.16.3 - September 10, 2020",
    2578i32 => "1.16.2 - August 11, 2020",
    2567i32 => "1.16.1 - June 24, 2020",
    2566i32 => "1.16 - June 23, 2020",

    // Version 1.15:
    2230i32 => "1.15.2 - January 21, 2020",
    2227i32 => "1.15.1 - December 17, 2019",
    2225i32 => "1.15 - December 10, 2019",

    // Version 1.14:
    1976i32 => "1.14.4 - July 19, 2019",
    1968i32 => "1.14.3 - June 24, 2019",
    1963i32 => "1.14.2 - May 27, 2019",
    1957i32 => "1.14.1 - May 13, 2019",
    1952i32 => "1.14 - April 23, 2019",

    // Version 1.13:
    1631i32 => "1.13.2 - October 22, 2018",
    1628i32 => "1.13.1 - August 22, 2018",
    1519i32 => "1.13 - July 18, 2018",

    // Version 1.12:
    1343i32 => "1.12.2 - September 18, 2017",
    1241i32 => "1.12.1 - August 3, 2017",
    1139i32 => "1.12 - June 7, 2017",

    // Version 1.11:
    922i32 => "1.11.2 - December 21, 2016",
    921i32 => "1.11.1 - December 20, 2016",
    819i32 => "1.11 - November 14, 2016",

    // Version 1.10:
    512i32 => "1.10.2 - June 23, 2016",
    511i32 => "1.10.1 - June 22, 2016",
    510i32 => "1.10 - June 8, 2016",

    // Version 1.9:
    184i32 => "1.9.4 - May 10, 2016",
    183i32 => "1.9.3 - May 10, 2016",
    176i32 => "1.9.2 - March 30, 2016",
    175i32 => "1.9.1 - March 30, 2016",
    169i32 => "1.9 - February 29, 2016",

    // April Fools versions:
    3824i32 => "April Fools version 2024: Java Edition 24w14potato",
    3444i32 => "April Fools version 2023: Java Edition 23w13a_or_b",
    3076i32 => "April Fools version 2022: Java Edition 22w13oneBlockAtATime",
    2522i32 => "April Fools version 2020: Java Edition 20w14∞",
    1943i32 => "April Fools version 2019: Java Edition 3D Shareware v1.34",
    173i32 => "April Fools version 2016: Java Edition 1.RV-Pre1",
};

/* https://minecraft.wiki/w/.minecraft */
#[cfg(target_os = "windows")]
const MINECRAFT_PATH: &'static str = "%APPDATA%\\.minecraft";
#[cfg(target_os = "macos")]
const MINECRAFT_PATH: &'static str = "~/Library/Application Support/minecraft";
#[cfg(target_os = "linux")]
const MINECRAFT_PATH: &'static str = "~/.minecraft";

fn unix_to_str(unix_timestamp_in_ms: i64) -> String {
    let system_time = UNIX_EPOCH + Duration::from_millis(unix_timestamp_in_ms as u64);
    let date_time = DateTime::<Utc>::from(system_time);
    date_time.format("%Y-%m-%d %H:%M:%S").to_string()
}

lazy_static::lazy_static! {
    static ref USERNAME_CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[derive(Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
}
// ...as returned by the https://api.minecraftservices.com/minecraft/profile/lookup/<UUID> API!

fn uuid_to_uname(uuid: &str) -> Result<String, String> {  // ToDo: allow user to disable this feature using --no-mojang-api and to alter the timeout using --mojang-api-timeout
    // https://minecraft.wiki/w/Mojang_API#Query_player's_username
    //   API: https://api.minecraftservices.com/minecraft/profile/lookup/<UUID>
    // where <UUID> must be without the minuses ("-")!
    //
    // Example GET request:
    //   https://api.minecraftservices.com/minecraft/profile/lookup/afe703c40a8f4b448301974a3305820d
    //
    // Example JSON response:
    //   {
    //     "id" : "afe703c40a8f4b448301974a3305820d",
    //     "name" : "horstder2te"
    //   }

    // Remove dashes from the UUID:
    let uuid_no_dashes: String = uuid.replace("-", "");

    // Check the cache first:
    {
        let cache = USERNAME_CACHE.lock().unwrap();
        if let Some(username) = cache.get(&uuid_no_dashes) {
            return Ok(username.clone());
        }
    }

    // Construct the API URL:
    let url = format!("https://api.minecraftservices.com/minecraft/profile/lookup/{}", uuid_no_dashes);

    // Create a blocking HTTP client with a timeout:
    let client = Client::builder()
        .timeout(Duration::from_secs(3)) // Set timeout to 3 seconds
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    // Send the GET request:
    let response = client.get(&url).send()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    // Sleep after each GET request to avoid an "HTTP 429 Too Many Requests":
    thread::sleep(Duration::from_millis(1000));

    // Check for successful response:
    if response.status().is_success() {
        // Parse the JSON response:
        let profile: MinecraftProfile = response.json()
            .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

        // Update the cache:
        {
            let mut cache = USERNAME_CACHE.lock().unwrap();
            cache.insert(uuid_no_dashes.clone(), profile.name.clone());
        }

        // Return the username:
        Ok(profile.name)
    } else {
        Err(format!("Failed to fetch username for UUID {}: HTTP {}", uuid, response.status()))
    }
}

/// A Minecraft world is a folder that must at least contain a valid "level.dat" file.
struct MinecraftWorld {
    path: PathBuf,
    level_dat: LevelDat,  // /world/level.dat file
    player_dat: Vec<PlayerDat>, // /world/playerdata/*.dat files
}

impl Display for MinecraftWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("***** ***** {:?} ***** *****\n", self.path))?;
        if let Some(data_version) = self.level_dat.data_version {
            if data_version > *DATA_VERSIONS.keys().max().unwrap() {
                f.write_str(&format!("Minecraft version: {} (>{})\n", data_version, DATA_VERSIONS.get(DATA_VERSIONS.keys().max().unwrap()).unwrap()))?;
            } else {
                f.write_str(&format!("Minecraft version: {} ({})\n", data_version, DATA_VERSIONS.get(&data_version).unwrap_or(&"???")))?;
            }
        } else {
            f.write_str(&"Minecraft version: <1.9\n".to_string())?;
        }
        f.write_str(&format!("Seed: {}\n", self.level_dat.random_seed.map(|s| s.to_string()).unwrap_or("???".to_string())))?;
        f.write_str(&format!("Last played: {} (UNIX: {})\n", unix_to_str(self.level_dat.last_played), self.level_dat.last_played))?;
        f.write_str(&format!("Modified: {}\n", "todo"))?; // TODO
        f.write_str(&format!("Size: {}\n", "todo"))?; // TODO
        f.write_str(&format!("Ticks passed: {} (~{:.2} hours)\n", self.level_dat.time, (self.level_dat.time as f64)/(20.0*3600.0)))?; // TODO: remove?!
        f.write_str(&format!("In-game days passed: {}\n", self.level_dat.day_time as f64 / 24000.0))?;
        f.write_str(&format!("Current time: {} (0 = sunrise, 6000 = midday, 12000 = sunset, 18000 = midnight)\n", self.level_dat.day_time % 24000))?;
        f.write_str(&format!("Difficulty: {} (0 = Peaceful, 1 = Easy, 2 = Normal, 3 = Hard)\n", self.level_dat.difficulty))?;
        f.write_str(&format!("Players: {}\n", self.player_dat.len()))?;
        for player in self.player_dat.iter() {
            f.write_str(&format!("    - {} ({}) @ x={:.2}, y={:.2}, z={:.2} (Health: {:.2}, Food: {})\n", player.uuid, uuid_to_uname(&player.uuid).unwrap_or("???".to_string()), player.pos.0, player.pos.1, player.pos.2, player.health, player.food_level))?;
        }
        Ok(())
    }
}

impl MinecraftWorld {
    fn new(level_dat: &Path) -> Result<Self, NBTError> {
        let parent_dir = level_dat.parent().map(PathBuf::from).ok_or(NBTError { msg: format!("{:?} has no parent", level_dat)})?;
        Ok(
            Self {
                path: parent_dir.clone(),
                level_dat: LevelDat::new(level_dat)?,
                player_dat: PlayerDat::for_each_dat_file_in(&parent_dir.join("playerdata")),
            }
        )
    }
}

/// Contains information from the "level.dat" file,
/// cf. https://minecraft.wiki/w/Java_Edition_level_format#level.dat_format
/// The data is stored in the so called "NBT" format,
/// cf. https://minecraft.wiki/w/NBT_format
/// Each Minecraft world folder must contain such a "level.dat" file.
struct LevelDat {
    day_time: i64, // "DayTime": 1 day = 24000, does not(!) reset to zero
    difficulty: i8, // "Difficulty"
    data_version: Option<i32>, // "DataVersion": https://minecraft.wiki/w/Data_version (MC v1.9+)
    last_played: i64, // "LastPlayed": "The Unix time in milliseconds when the level was last loaded."
    random_seed: Option<i64>, // "RandomSeed": "The random level seed used to generate consistent terrain."
    time: i64, // "Time": "The number of ticks since the start of the level."
}

impl LevelDat {
    fn new(level_dat: &Path) -> Result<Self, NBTError> {
        let nbt_file: NBTFile = NBTFile::new(level_dat)?;
        Ok(
            Self {
                day_time: nbt_file.get_long("DayTime")?,
                difficulty: nbt_file.get_byte("Difficulty")?,
                data_version: nbt_file.get_int("DataVersion").ok(),
                last_played: nbt_file.get_long("LastPlayed")?,
                random_seed: nbt_file.get_long("RandomSeed").ok(),
                time: nbt_file.get_long("Time")?,
            }
        )
    }
}

/// A <player>.dat file stores the state of individual players,
/// cf. https://minecraft.wiki/w/Player.dat_format
/// Just like the "level.dat" file, it is also stored in "NBT" format,
/// cf. https://minecraft.wiki/w/NBT_format
/// The /world/playerdata/ folder contains a <player>.dat file for each player.
struct PlayerDat {
    uuid: String, // extracted from the file name, e.g. "afe703c4-0a8f-4b44-8301-974a3305820d.dat"
    health: f32, // "Health"
    food_level: i32, // "foodLevel"
    pos: (f64, f64, f64), // "Pos": "List of 3 doubles describing the current X, Y, and Z position (coordinates) of the entity."
}

impl PlayerDat {
    fn new(player_dat: &Path) -> Result<Self, NBTError> {
        let nbt_file: NBTFile = NBTFile::new(player_dat)?;
        Ok(
            Self {
                uuid: player_dat.file_name().unwrap().to_str().unwrap().strip_suffix(".dat").unwrap().to_string(),
                health: nbt_file.get_float("Health")?,
                food_level: nbt_file.get_int("foodLevel")?,
                pos: nbt_file.get_double_triplet("Pos")?,
            }
        )
    }

    fn for_each_dat_file_in(folder: &Path) -> Vec<Self> {
        let mut result: Vec<Self> = Vec::new();
        let files = fs::read_dir(folder).unwrap();
        for file in files {
            if let Ok(file) = file {
                if file.file_name().into_string().unwrap().ends_with(".dat") {
                    if let Ok(player_dat) = PlayerDat::new(&file.path()) {
                        result.push(player_dat);
                    }
                }
            }
        }
        return result;
    }
}

#[derive(Debug)]
struct NBTError {
    msg: String,
}

/// Cf. https://minecraft.wiki/w/NBT_format
struct NBTFile {
    data: Vec<u8>,
}

impl NBTFile {
    fn new(path: &Path) -> Result<Self, NBTError> {
        let file = File::open(path).map_err(|_e| NBTError {msg: format!("Could not open file {:?}", path)})?;
        let file = BufReader::new(file);
        let mut file = GzDecoder::new(file);
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(|_e| NBTError {msg: format!("Could not read file {:?}", path)})?;

        Ok(
            Self {
                data: bytes,
            }
        )
    }

    fn get_byte(&self, name: &str) -> Result<i8, NBTError> {
        const SIZE: usize = std::mem::size_of::<i8>();
        for i in 0..=self.data.len()-name.len()-SIZE {
            if self.data[i..i+name.len()].to_vec() == name.as_bytes() {
                return Ok(i8::from_be_bytes([
                    self.data[i+name.len()],
                ]))
            }
        }
        Err(NBTError {msg: format!("'{}' not found", name)})
    }

    fn get_int(&self, name: &str) -> Result<i32, NBTError> {
        const SIZE: usize = std::mem::size_of::<i32>();
        for i in 0..=self.data.len()-name.len()-SIZE {
            if self.data[i..i+name.len()].to_vec() == name.as_bytes() {
                return Ok(i32::from_be_bytes([
                    self.data[i+name.len()],
                    self.data[i+name.len()+1],
                    self.data[i+name.len()+2],
                    self.data[i+name.len()+3],
                ]))
            }
        }
        Err(NBTError {msg: format!("'{}' not found", name)})
    }

    fn get_long(&self, name: &str) -> Result<i64, NBTError> {
        const SIZE: usize = std::mem::size_of::<i64>();
        for i in 0..=self.data.len()-name.len()-SIZE {
            if self.data[i..i+name.len()].to_vec() == name.as_bytes() {
                return Ok(i64::from_be_bytes([
                    self.data[i+name.len()],
                    self.data[i+name.len()+1],
                    self.data[i+name.len()+2],
                    self.data[i+name.len()+3],
                    self.data[i+name.len()+4],
                    self.data[i+name.len()+5],
                    self.data[i+name.len()+6],
                    self.data[i+name.len()+7],
                ]))
            }
        }
        Err(NBTError {msg: format!("'{}' not found", name)})
    }

    fn get_float(&self, name: &str) -> Result<f32, NBTError> {
        const SIZE: usize = std::mem::size_of::<f32>();
        for i in 0..=self.data.len()-name.len()-SIZE {
            if self.data[i..i+name.len()].to_vec() == name.as_bytes() {
                return Ok(f32::from_be_bytes([
                    self.data[i+name.len()],
                    self.data[i+name.len()+1],
                    self.data[i+name.len()+2],
                    self.data[i+name.len()+3],
                ]))
            }
        }
        Err(NBTError {msg: format!("'{}' not found", name)})
    }

    fn get_double_triplet(&self, name: &str) -> Result<(f64, f64, f64), NBTError> {
        Ok((0.0, 0.0, 0.0))  // TODO
    }
}

fn main() {
    // (1.) Parse command line args or use default values:
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 0 {
        #[cfg(unix)]
        args.push(shellexpand::tilde(MINECRAFT_PATH).to_string());
        #[cfg(windows)]
        args.push(MINECRAFT_PATH.to_string());

        #[cfg(unix)]
        args.push(shellexpand::tilde("~").to_string());

        #[cfg(unix)]
        args.push("/".to_string());
        #[cfg(windows)]
        args.push("C:\\".to_string());
    }

    // (2.) Iterate through each given folder and print each MinecraftWorld found, store paths
    //      of MinecraftWorlds already found to avoid printing them twice when multiple paths
    //      were given (e.g., first "~/.minecraft" and then "/"):
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut min_version: i32 = i32::MAX;
    let mut max_version: i32 = i32::MIN;
    for dir in args {
        println!();
        println!("Walking through {} ...", dir);
        println!();
        for level_dat_file in WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.file_name() == "level.dat")
        {

            // Ignore this level.dat file if it has already been processed before:
            let path: &Path = level_dat_file.path();
            let path_buf: PathBuf = path.to_path_buf();
            if paths.contains(&path_buf) {
                continue;
            } else {
                paths.push(path_buf);
            }

            // Try to parse the level.dat and associated files and print Minecraft world info,
            //   otherwise print an error message:
            println!();
            match MinecraftWorld::new(path) {
                Ok(mc_world) => {
                    println!("{}", mc_world);
                    if let Some(version) = mc_world.level_dat.data_version {
                        if version < min_version {
                            min_version = version;
                        }
                        if version > max_version {
                            max_version = version;
                        }
                    }
                }
                Err(err) => {
                    println!("{:?} is invalid: {:?}", path, err.msg);
                }
            }
            println!();

        }
    }

    println!("Done. {} Minecraft worlds were found.", paths.len());
    println!();
    println!("Highest version encountered: {} ({})", max_version, DATA_VERSIONS.get(&max_version).unwrap_or(&"???"));
    println!("Lowest >=1.9 version encountered: {} ({})", min_version, DATA_VERSIONS.get(&min_version).unwrap_or(&"???"));
    println!();
    // TODO: print out some final statistics (like oldest world, largest world, longest play time, ...)
}
