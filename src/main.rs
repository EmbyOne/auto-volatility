use structopt::StructOpt;
use std::thread;
use std::process::Command;
use std::str::from_utf8;
use regex::Regex;

//QUEUE SHIT BECAUSE PUBLISHED ONES SUCK ASS
//nvm i was just dumb but this is fine
struct Queue<T> {
    queue: Vec<T>,
}

impl<T> Queue<T> {
    fn new() -> Self {
      Queue { queue: Vec::new() }
    }
    
    fn enqueue(&mut self, item: T) {
      self.queue.push(item)
    }
  
    fn dequeue(&mut self) -> T {
      self.queue.remove(0)
    }
}

//CLI arguments
#[derive(StructOpt)]
struct Cli {
    //memory file to analyze
    memfile: String,
    //Output directory name
    outdir: String,
}

//Add a paramater for choosing threads later
const THREADS: usize = 8;

//Get profile of memory file 
fn getProfile(file: &str) -> String {
    let output = Command::new("vol")
        //don't even ask.
        //.args([("-f ".to_owned() + &file), "imageinfo".to_owned()])
        .arg(format!("-f {} imageinfo", file))
        .output()
        .expect("Failed to get profile.");
    
    let result = from_utf8(output.stdout.as_slice()).unwrap();
    
    //Getting profile via regex
    let re = Regex::new(r":\s([a-zA-Z0-9-.]*),").unwrap();
    if re.is_match(result) {
        let profile_index = re.find(result).unwrap();
        let profile = &result[profile_index.start()..profile_index.end()][2..].replace(",", "").replace("(", "");
        return profile.to_string();
    } else {
        println!("No profile found, please specify manually.");
        std::process::exit(1);
    }
}

fn main() {

    //Add the rest of the plugins later !
    //Creating a queue from the list of plugins
    let plugins = ["amcache", "auditpol", "cachedump", "clipboard", "cmdline", "cmdscan", "connections", "connscan", "consoles", "deskscan", "devicetree", "dlllist",
    "envars", "getservicesids", "handles", "hibinfo", "hivelist", "hivescan", "iehistory", "ldrmodules", "lsadump", "malfind", "mbrparser", "memmap", "mftparser", "modules", "notepad", 
    "privs", "pslist", "psscan", "pstree", "psxview", "qemuinfo", "servicediff", "sessions", "sockets", "sockscan", "ssdt", "strings", "svcscan", "symlinkscan", "thrdscan", "verinfo", "windows", "wintree"];    
    let mut q: Queue<&str> = Queue::new();
    for i in plugins {
        q.enqueue(i);
    }

    //Get args (this exists because there will be more args later)
    let args = Cli::from_args();
    let memfile = &args.memfile;
    let outdir = &args.outdir;

    //force get profile name, later can be specified in args manually
    let profile = getProfile(memfile);


    //Set up handler for threads
    let mut handles = vec![];

    for i in 0..THREADS {
        let plugin = q.dequeue();
        let plugindir = format!("{}/{}" ,outdir, plugin);
        let mut argline= "".to_owned();

        //To dump or not to dump (writes stdout to dir either way, but dumps extra stuff as well if plugin supplies)
        if plugin.contains("dump") || plugin == "servicediff" {
            //argvec = vec![("-f ".to_owned() + &memfile), ("--profile ".to_owned() + &profile), plugin.to_owned(), ("--dump-dir=".to_owned() + plugindir)];
            argline = format!("-f {} --profile={} {} --dump-dir={}", memfile, profile, plugin, plugindir)
        } else {
            //argvec = vec![("-f ".to_owned() + &memfile), ("--profile ".to_owned() + &profile), plugin.to_owned()];
            argline = format!("-f {} --profile={} {}", memfile, profile, plugin);
        }
        //threading o.0
        let handle = thread::spawn(move|| {

            //let mut argvec = vec![];
            
            //Check presence of output directory and create if needed
            if std::path::Path::new(&plugindir).exists() == false {
                std::fs::create_dir_all(&plugindir);
            } 

            let output = Command::new("vol")
                //don't even ask.
                //.args(argvec)
                .arg(argline)
                .output()
            	.expect("Failed to execute vol");

            let result = from_utf8(output.stdout.as_slice()).unwrap();
            //Write output to file
            std::fs::write(format!("{}.txt", plugindir), result)
        });

        //Add this thread to the vector(list) of threads
        handles.push(handle);
    }
    //Let all the threads finish executing before closing the program
    for i in handles {
        i.join().unwrap();
    }
}
