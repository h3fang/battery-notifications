use std::{collections::HashMap, io, process::Command};

use mio::{Events, Interest, Poll, Token};

#[derive(PartialEq, Clone)]
enum State {
    Others,
    Discharging,
    DischargingLow,
    DischargingCritical,
    Charging,
    ChargingHigh,
}

const HIGH_THRESHOLD: u8 = 80;
const LOW_THRESHOLD: u8 = 30;
const CRITICAL_THRESHOLD: u8 = 10;

fn poll(mut socket: udev::MonitorSocket) -> io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);

    poll.registry().register(
        &mut socket,
        Token(0),
        Interest::READABLE | Interest::WRITABLE,
    )?;

    let mut states = HashMap::new();

    loop {
        poll.poll(&mut events, None)?;
        for event in &events {
            if event.token() == Token(0) && event.is_writable() {
                socket.iter().for_each(|x| process_event(x, &mut states));
            }
        }
    }
}

fn notify(urgency: &str, body: &str) {
    Command::new("notify-send")
        .args(["-u", urgency, "Battery Notifications", body])
        .output()
        .unwrap();
}

fn process_event(event: udev::Event, states: &mut HashMap<String, State>) {
    if event.attribute_value("type").unwrap() != "Battery" {
        return;
    }

    let bat_name = event.sysname().to_str().unwrap();
    let status = event.attribute_value("status").unwrap().to_str().unwrap();
    let state = states.entry(bat_name.to_string()).or_insert(State::Others);
    let p = event
        .attribute_value("capacity")
        .unwrap()
        .to_string_lossy()
        .parse::<u8>()
        .unwrap();
    match status {
        "Discharging" => {
            if p <= CRITICAL_THRESHOLD {
                if *state != State::DischargingCritical {
                    *state = State::DischargingCritical;
                    notify(
                        "critical",
                        &format!("Battery {} is critically low.", bat_name),
                    );
                }
            } else if p <= LOW_THRESHOLD {
                if *state != State::DischargingLow {
                    *state = State::DischargingLow;
                    notify("critical", &format!("Battery {} is too low.", bat_name));
                }
            } else if *state != State::Discharging {
                *state = State::Discharging;
                notify("normal", &format!("Battery {} is discharging.", bat_name));
            }
        }
        "Charging" => {
            if p >= HIGH_THRESHOLD {
                if *state != State::ChargingHigh {
                    *state = State::ChargingHigh;
                    notify("critical", &format!("Battery {} is too full.", bat_name));
                }
            } else if *state != State::Charging {
                *state = State::Charging;
                notify("normal", &format!("Battery {} is charging.", bat_name));
            }
        }
        _ => {
            *state = State::Others;
        }
    }
}

fn main() -> std::io::Result<()> {
    let socket = udev::MonitorBuilder::new()?
        .match_subsystem("power_supply")?
        .listen()?;
    poll(socket)?;
    Ok(())
}
