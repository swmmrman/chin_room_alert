use reqwest;
use std::process;
use std::{io::Write, process::exit};
use std::thread::sleep;
use std::time::Duration;
use rppal::{gpio,pwm};


fn main() {
    let buzzer_open = 
        // pwm::Pwm::with_frequency(
        //     pwm::Channel::Pwm0,
        //     7400.0f64,
        //     50.0f64,
        //     pwm::Polarity::Normal,
        //     false,
        pwm::Pwm::new(pwm::Channel::Pwm0);
    //);
    let buzzer = match buzzer_open {
        Ok(p) => p,
        Err(e) => {
            println!("PWM Error:{}", e);
            exit(1);
        },
    };
    buzzer.set_frequency(1000.2f64, 0.5f64).unwrap();
    buzzer.set_polarity(pwm::Polarity::Normal).unwrap();

    loop {
        let ans = match reqwest::blocking::get("http://192.168.1.134/temp_in.txt") {
            Ok(r) =>  match r.text() {
                Ok(t) => t,
                Err(_) => "8888".to_owned(),
            }
            Err(e) => { 
                eprintln!("{:?}", e);
                "404".to_owned()
            }
        };
        let mut outfile = std::fs::OpenOptions::new()
            .write(true)
            .create(false)
            .open("/tmp/ledpipe")
            .unwrap();
        match outfile.write(format!("{}\n",ans).as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                println!("Pipe failure: {}", e);
                process::exit(1)
            }
        }
        match outfile.flush() {
            Ok(_) => (),
            Err(e) => {
                println!("Flush failure: {}, Call the plumber.", e)
            }
        }
        drop(outfile);
        //println!("{}", ans);
        // match buzzer.enable() {
        //     Ok(_) => (),
        //     Err(_) => (),
        // }
        // sleep(Duration::from_millis(10000));
        // match buzzer.disable() {
        //     Ok(_) => (),
        //     Err(_) => (),
        // }
        sleep(Duration::from_millis(2000));
    }
}
