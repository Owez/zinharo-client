use bzip2::read::BzDecoder;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use std::{env, process, thread, time};
use zinharo_rs::{ZinharoAccess, ZinharoError, ZinharoQueuedJob};

/// Graffiti hackerman header message
const HEADER_MSG: &str = " _______       _                        _____ _ _            _   \n|___  (_)     | |                      / ____| (_)          | |  \n   / / _ _ __ | |__   __ _ _ __ ___   | |    | |_  ___ _ __ | |_ \n  / / | | '_ \\| '_ \\ / _` | '__/ _ \\  | |    | | |/ _ \\ '_ \\| __|\n / /__| | | | | | | | (_| | | | (_) | | |____| | |  __/ | | | |_ \n/_____|_|_| |_|_| |_|\\__,_|_|  \\___/   \\_____|_|_|\\___|_| |_|\\__|\n\n";

/// Sleeps for x seconds, used because of rusts bad std
fn sleep_sec(secs: u64) {
    let sleep_dur = time::Duration::from_secs(secs);
    thread::sleep(sleep_dur);
}

/// Signs up to Zinharo with new client, **should be used wisely**
fn signup(username: &str, password: &str) -> ZinharoAccess {
    match ZinharoAccess::signup(username, password) {
        Ok(access) => access,
        Err(ZinharoError::Ratelimited) => {
            eprintln!("Ratelimited when attempting signup, retrying in 1 hour..");

            sleep_sec(60 * 60);
            signup(username, password)
        }
        Err(ZinharoError::UsernameTaken) => {
            eprintln!("Signup username taken, please choose another one or toggle `ZINHARO_SIGNUP` off if it is your account!");
            process::exit(1);
        }
        Err(_) => {
            eprintln!("Unknown error whilst trying to signup, this shouldn't happen!");
            process::exit(1);
        }
    }
}

/// Gets enviroment variables and logs in
fn login_startup() -> ZinharoAccess {
    let username = match env::var("ZINHARO_USERNAME") {
        Ok(x) => x,
        Err(_) => {
            eprintln!("Please supply the `ZINHARO_USERNAME` enviroment variable!");
            process::exit(1);
        }
    };

    let password = match env::var("ZINHARO_PASSWORD") {
        Ok(x) => x,
        Err(_) => {
            eprintln!("Please supply the `ZINHARO_PASSWORD` enviroment variable!");
            process::exit(1);
        }
    };

    match ZinharoAccess::login(&username, &password) {
        Ok(access) => access,
        Err(ZinharoError::Ratelimited) => {
            eprintln!("Ratelimited when trying to login, retrying in 30 seconds..");

            sleep_sec(30);
            login_startup()
        }
        Err(ZinharoError::ReqwestError(_)) => {
            eprintln!("Could not connect to Zinaro API, retrying in 30 seconds..");

            sleep_sec(30);
            login_startup()
        }
        Err(ZinharoError::ApiVersionInadequate) => {
            eprintln!("This client is critically out of date, please update!");
            process::exit(1);
        }
        Err(ZinharoError::BadCredentials) => {
            eprintln!("Username or password invalid, will attempt signup if the env-var\n`ZINHARO_SIGNUP` is set!");

            if env::var("ZINHARO_SIGNUP").is_ok() {
                signup(&username, &password) // return this
            } else {
                process::exit(1);
            }
        }
        Err(ZinharoError::FirewallBlock) => {
            eprintln!("You have been temporarily blocked from the Zinharo API by\nCloudflare or a local firewall. Please ensure you are not routing\nthrough tor!");
            process::exit(1);
        }
        Err(_) => {
            eprintln!("Unknown fatal error when logging in!");
            process::exit(1);
        }
    }
}

/// Saves byte-formatted bzip2 to a given wordlist path while decoding it
fn save_bzip2(compressed: bytes::Bytes, wordlist_path: PathBuf) {
    let mut decompressor = BzDecoder::new(&compressed[..]);
    let mut contents: Vec<u8> = Vec::new();

    match decompressor.read_to_end(&mut contents) {
        Ok(_) => (),
        Err(_) => {
            eprintln!("Error whilst decompressing standardized wordlist, this shouldn't happen!");
            process::exit(1);
        }
    }

    let fileperm_error = ", ensure file permissions are correct!";

    let mut wordlist_file = match File::create(wordlist_path) {
        Ok(x) => x,
        Err(_) => {
            eprintln!("Could not create file to save wordlist{}", fileperm_error);
            process::exit(1);
        }
    };

    match wordlist_file.write_all(&contents) {
        Ok(_) => (),
        Err(_) => {
            eprintln!("Could not save wordlist contents to file{}", fileperm_error);
            process::exit(1);
        }
    };
}

/// Gets the `rockyou.txt` wordlist from an external source and saves it to
/// local `wordlist.txt` path
fn get_wordlist(access: &ZinharoAccess) -> PathBuf {
    let wordlist_path = PathBuf::from("./wordlist.txt");

    if !wordlist_path.exists() {
        println!("Downloading wordlist..");

        let wordlist_url = "http://downloads.skullsecurity.org/passwords/cain.txt.bz2";
        let resp = match access.client.get(wordlist_url).send() {
            Ok(x) => x,
            Err(_) => {
                eprintln!("Fatal whilst downloading wordlist, `skullsecurity.org` may be down!");
                process::exit(1);
            }
        };

        let compressed = match resp.bytes() {
            Ok(x) => x,
            Err(_) => {
                eprintln!("Could not download compressed wordlist body, may have been timed-out!");
                process::exit(1);
            }
        };

        save_bzip2(compressed, PathBuf::clone(&wordlist_path));
    }

    wordlist_path
}

/// In a seperate function for error handling
fn report_job(access: &ZinharoAccess, job: ZinharoQueuedJob, info: Option<&str>) {
    let cont_message = ", continuing anyway..";

    match job.report(access, info) {
        Ok(_) => (),
        Err(ZinharoError::Ratelimited) => eprintln!("Ratelimited when reporting{}", cont_message),
        Err(_) => eprintln!("Could not report job{}", cont_message),
    }
}

/// Opens a file with the wifi password in and uploads it to Zinharo
fn upload_job(access: &ZinharoAccess, job: ZinharoQueuedJob, path: PathBuf) -> Result<(), ()> {
    if !path.exists() {
        eprintln!("The password file aircrack-ng dumped to does not exist, this shouldn't happen!");
        process::exit(1);
    }

    let mut file = match File::open(PathBuf::clone(&path)) {
        Ok(x) => x,
        Err(_) => {
            eprintln!("Error whilst opening password file, please check zinharo has rights to read files!");
            process::exit(1);
        }
    };

    let mut contents = String::new();

    match file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(_) => {
            eprintln!("Error while reading password file, password possibly invalid utf-8!");
            return Err(());
        }
    };

    match job.submit(access, &contents) {
        Ok(()) => Ok(()),
        Err(ZinharoError::Ratelimited) => {
            eprintln!("Ratelimited whilst submitting job, retrying in 30 seconds..");

            sleep_sec(30);
            upload_job(access, job, path) // I know this is expensive
        }
        Err(_) => {
            eprintln!("Unknown error whilst submitting job, retrying in 10 seconds..");

            sleep_sec(10);
            upload_job(access, job, path) // I know this is expensive (x2)
        }
    }
}

/// Starts to crack given cap file inside job
fn start_job(
    access: &ZinharoAccess,
    job: ZinharoQueuedJob,
    wordlist_path: PathBuf,
) -> Option<PathBuf> {
    let output_path = PathBuf::from("./out.txt");
    let cap_path = PathBuf::from("./inprogress.cap");

    match job.dump_cap(PathBuf::clone(&cap_path)) {
        Ok(_) => (),
        Err(e) => {
            eprintln!(
                "Could not save job #{} to file: '{:?}', reporting job!",
                job.id, e
            );
            report_job(
                access,
                job,
                Some("Could not save to file, possibly invalid cap"),
            );
            return None;
        }
    };

    let output_path_str = PathBuf::clone(&output_path)
        .into_os_string()
        .into_string()
        .unwrap();
    let cap_path_str = cap_path.into_os_string().into_string().unwrap();
    let wordlist_path_str = wordlist_path.into_os_string().into_string().unwrap();

    let output = Command::new("aircrack-ng")
        .args(&[
            &cap_path_str,
            "-w",
            &wordlist_path_str,
            "-l",
            &output_path_str,
        ])
        .output();

    match output {
        Ok(cmd) => {
            if cmd.status.success() {
                if output_path.exists() {
                    println!("Found password, uploading..");

                    match upload_job(access, job, PathBuf::clone(&output_path)) {
                        Ok(_) => (),
                        Err(_) => return None,
                    };

                    Some(output_path)
                } else {
                    eprintln!("No password found, reporting..");
                    report_job(
                        &access,
                        job,
                        Some("Could not crack using standardised wordlist"),
                    );
                    None
                }
            } else {
                eprintln!("Could not crack due to underlying error in aircrack-ng!");
                process::exit(1);
            }
        }
        Err(_) => {
            eprintln!("Could not crack due to underlying error when calling aircrack-ng, maybe give zinharo admin rights?");
            process::exit(1);
        }
    }
}

fn main() {
    println!("{}\n            The automated Zinharo.com cracking client\n=================================================================", HEADER_MSG);

    let access = login_startup();
    let wordlist_path = get_wordlist(&access);

    println!("Client launched successfully!");

    loop {
        let found_job = match ZinharoQueuedJob::new(&access) {
            Ok(x) => x,
            Err(ZinharoError::Ratelimited) => {
                eprintln!("Ratelimited whilst fetching job, retrying in 1 min..");
                sleep_sec(60);
                continue;
            }
            Err(ZinharoError::NoJobsAvailable) => {
                eprintln!("No jobs currently available, asking again in 2 mins..");
                sleep_sec(120);
                continue;
            }
            Err(e) => {
                eprintln!(
                    "Unknown error whilst fetching job, retrying in 30 seconds..\n{:?}",
                    e
                );
                sleep_sec(30);
                continue;
            }
        };

        println!("Fetched job #{}, cracking..", found_job.id);
        match start_job(&access, found_job, PathBuf::clone(&wordlist_path)) {
            Some(_) => {
                println!("Fetching new job..");
            }
            None => {
                eprintln!("Fetching new job in 30 secs..");
                sleep_sec(30); // 30 second timeout so hopefully new job comes
            }
        }
    }
}
