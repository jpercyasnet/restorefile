use iced::widget::{button, column, row, text_input, text, Space, checkbox, progress_bar};
use iced::{Alignment, Element, Task, Color, Size};
use iced::theme::{Theme};
use iced::Subscription;
use iced::window;
use iced::futures;
use iced::event::{self, Event};
use futures::channel::mpsc;
extern crate chrono;
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::time::Duration as timeDuration;
use std::time::Instant as timeInstant;
use std::thread::sleep;
use chrono::Local;
use std::path::{Path};
use std::process::Command as stdCommand;

use eject::{device::{Device, DriveStatus}, discovery::cd_drives};

mod get_winsize;
mod inputpress;
mod execpress;
mod sortin;
use get_winsize::get_winsize;
use inputpress::inputpress;
use execpress::execpress;
use sortin::sortin;

pub fn main() -> iced::Result {

     let mut widthxx: f32 = 500.0;
     let mut heightxx: f32 = 300.0;
     let (errcode, errstring, widtho, heighto) = get_winsize();
     if errcode == 0 {
         widthxx = widtho as f32 - 20.0;
         heightxx = heighto as f32 - 75.0;
         println!("{}", errstring);
     } else {
         println!("**ERROR {} get_winsize: {}", errcode, errstring);
     }
     println!("widthxx: -{}-  heightxx: -{}- ", widthxx, heightxx);
     iced::application(Restorefiles::title, Restorefiles::update, Restorefiles::view)
        .window(window::Settings {
            max_size: Some(Size::new(widthxx, heightxx)),
            decorations: true,
            ..Default::default()
        })
        .window_size((widthxx, heightxx))
        .theme(Restorefiles::theme)
        .subscription(Restorefiles::subscription)
        .run_with(Restorefiles::new)
}

struct Restorefiles {
    sortinname: String,
    bkrefname: String,
    listrefname: Vec<String>,
    listindex: u64,
    bklabel: String,
    bkpath: String,
    mess_color: Color,
    msg_value: String,
    altname: String,
    alt_bool: bool,
    targetdir: String,
    do_progress: bool,
    progval: f64,
    screenwidth: f32,
    screenheight: f32,
    tx_send: mpsc::UnboundedSender<String>,
    rx_receive: mpsc::UnboundedReceiver<String>,
}

#[derive(Debug, Clone)]
enum Message {
    SortinPressed,
    BkPressed,
    Alt(bool),
    TargetdirPressed,
    AltnameChanged(String),
    FirstPressed,
    NextPressed,
    ExecPressed,
    ExecxFound(Result<Execx, Error>),
    ProgressPressed,
    ProgRtn(Result<Progstart, Error>),
    Size(Size),

}

impl Restorefiles {
     fn new() -> (Self, Task<Message>) {
        let (tx_send, rx_receive) = mpsc::unbounded();
        ( Self { sortinname: "--".to_string(), bklabel: "--".to_string(), bkpath: "--".to_string(), msg_value: "no message".to_string(),
               targetdir: "--".to_string(), mess_color: Color::from([0.0, 0.0, 1.0]), alt_bool: false, altname: "--".to_string(), 
               do_progress: false, progval: 0.0, tx_send, rx_receive,
               screenwidth: 999.0, screenheight: 999.0, bkrefname: "--".to_string(), listrefname: Vec::new(), listindex: 0,

          },
          Task::none()
        )
    }

    fn title(&self) -> String {
        String::from("Backup file list with md5sum -- iced")
    }

    fn update(&mut self, message: Message) -> Task<Message>  {
        match message {
            Message::SortinPressed => {
               self.mess_color = Color::from([1.0, 0.0, 0.0]);
               let (errcode, errstr, newinput, lrefname, lindex, llist) = sortin(self.sortinname.clone());
               self.msg_value = errstr.to_string();
               if errcode == 0 {
                   self.sortinname = newinput.to_string();
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
                   self.listindex = lindex;
                   self.listrefname = llist;
                   let numlist = self.listrefname.len();
                   self.bkrefname = lrefname;
                   self.msg_value = format!("Please mount {} which has {} files", self.bkrefname, numlist);
               }
               Task::none()
            }
            Message::BkPressed => {
               let mut n = 0;   
               let mut mountv = "--".to_string();
               for path in cd_drives() {
                  // Print the path
//                println!("{:?}", path);
                  let strpath = format!("{:?}", path);
                  // Access the drive
                  match Device::open(path.clone()){
                    Ok(drive) => {
                         n = n + 1;
                         self.msg_value = "able to open".to_string();
                         match drive.status() {
                           Ok(DriveStatus::Empty) =>
                              self.msg_value = "The tray is closed and no disc is inside".to_string(),
                           Ok(DriveStatus::TrayOpen) =>
                              self.msg_value = "The tray is open".to_string(),
                           Ok(DriveStatus::NotReady) =>
                              self.msg_value = "This drive is not ready yet".to_string(),
                           Ok(DriveStatus::Loaded) => {
                              self.msg_value = "There's a disc inside".to_string();
                              let mut line = String::new();
                              let mut linenum = 0;
                              let file = File::open("/proc/mounts").unwrap(); 
                              let mut reader = BufReader::new(file);
                              loop {
                                   match reader.read_line(&mut line) {
                                     Ok(bytes_read) => {
                                        // EOF: save last file address to restart from this address for next run
                                        if bytes_read == 0 {
                                            break;
                                        }
                                        let vecline: Vec<&str> = line.split(" ").collect();
                                        linenum = linenum + 1;
                                        let devname: String = vecline[0].to_string();
                                        let mountname = vecline[1].to_string();
                                        if devname.contains("/dev") {
//                                            println!("{} device: {} has mount of {}", linenum, devname, mountname);
                                            let strtrim: String = strpath[1..(strpath.len() -1)].to_string();
                                            if devname.contains(&strtrim) {
//                                                println!("found disc {} {} with mount of {}", strpath, strtrim, mountname);
                                                mountv = mountname;
                                            }
                                        }
                                        line.clear();
                                     }
                                     Err(err) => {
                                        self.msg_value = format!("error in read proc {} ", err);
                                        break;
                                     }
                                   }
                              }
                           }
                           Err(e) => {
                               self.msg_value = format!("error {} in status dvd", e);
                           }
                         }
                    }
                    Err(e) => {
                       self.msg_value = format!("error {} in retracting dvd", e);
                    }
                  }
               }
               self.mess_color = Color::from([1.0, 0.0, 0.0]);
               self.alt_bool = false;
               self.altname = "--".to_string();
               if n < 1 {
                   self.msg_value = "no dvd drives found".to_string();
               } else {
                   if mountv != "--" {
                       let veclabel: Vec<&str> = mountv.split("/").collect();
                       self.bkpath = mountv.clone();
                       self.bklabel = veclabel[veclabel.len() - 1].to_string();
                     
                   }
               }
               Task::none()
           }
            Message::AltnameChanged(value) => { self.altname = value; Task::none() }
            Message::Alt(picked) => {self.alt_bool = picked; Task::none()}
            Message::TargetdirPressed => {
               let (errcode, errstr, newinput) = inputpress(self.targetdir.clone());
               self.msg_value = errstr.to_string();
               if errcode == 0 {
                   self.targetdir = newinput.to_string();
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               }
               Task::none()
            }
            Message::FirstPressed => {
               if !Path::new(&self.sortinname).exists() {
                   self.msg_value = "sorted input file does not exist".to_string();
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               } else {
                   let file = File::open(self.sortinname.clone()).unwrap();
                   let mut reader = BufReader::new(file);
                   let mut linehd = String::new();
                   let mut linenum: u64 = 0;
                   let mut lrefname = "---".to_string();
                   let mut llist: Vec<String> = Vec::new();
                   let mut bolok = true;
                   loop {
                       match reader.read_line(&mut linehd) {
                            Ok(bytes_read) => {
                                if bytes_read == 0 {
                                    if linenum < 1 {
                                         self.msg_value = format!("error bytes_read == 0 for {}", self.sortinname);
                                         self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                         bolok = false;
                                    }
                                    break;
                                }
                                linenum = linenum + 1;
                                if linenum < 2 {
                                    let cnt = linehd.matches("|").count();
                                    if cnt < 7 {
                                        self.msg_value = format!("first line of sorted input file is not valid: {}", linehd);
                                        self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                        bolok = false;
                                        break;
                                    }
                                    let vecline: Vec<&str> = linehd.split("|").collect();
                                    llist.push(linehd.clone());
                                    lrefname = vecline[0].to_string();
                                } else {
                                    let veclinen: Vec<&str> = linehd.split("|").collect();
                                    let lrefnamen = veclinen[0].to_string();
                                    if lrefname == lrefnamen {
                                        llist.push(linehd.clone());
                                    } else {
                                        linenum = linenum - 1;
                                        break;
                                   }
                                }
                                linehd.clear();
                            }
                            Err(err) => {  
                                self.msg_value = format!("error of {} reading {}", err, self.sortinname);
                                self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                bolok = false;
                                break;
                            }
                       };
                   }
                   if bolok {
                       self.mess_color = Color::from([0.0, 1.0, 0.0]);
                       self.listindex = linenum;
                       self.listrefname = llist;
                       let numlist = self.listrefname.len();
                       self.bkrefname = lrefname;
                       self.msg_value = format!("Please mount {} which has {} files", self.bkrefname, numlist);
                   }
               }
               Task::none()
            }
            Message::NextPressed => {
               if !Path::new(&self.sortinname).exists() {
                   self.msg_value = "sorted input file does not exist".to_string();
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               } else {
                   let file = File::open(self.sortinname.clone()).unwrap();
                   let mut reader = BufReader::new(file);
                   let mut linehd = String::new();
                   let mut linenum: u64 = 0;
                   let mut lrefname = "---".to_string();
                   let mut llist: Vec<String> = Vec::new();
                   let mut bolok = true;
                   let mut bolend = false;
                   loop {
                       match reader.read_line(&mut linehd) {
                            Ok(bytes_read) => {
                                if bytes_read == 0 {
                                    bolend = true;
                                    if linenum < 1 {
                                         self.msg_value = format!("error bytes_read == 0 for {}", self.sortinname);
                                         self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                         bolok = false;
                                    }
                                    break;
                                }
                                linenum = linenum + 1;
                                if linenum >= (self.listindex + 1) {
                                    if linenum == (self.listindex + 1) {
                                        let vecline: Vec<&str> = linehd.split("|").collect();
                                        llist.push(linehd.clone());
                                        lrefname = vecline[0].to_string();
                                    } else {
                                        let veclinen: Vec<&str> = linehd.split("|").collect();
                                        let lrefnamen = veclinen[0].to_string();
                                        if lrefname == lrefnamen {
                                            llist.push(linehd.clone());
                                        } else {
                                           linenum = linenum - 1;
                                           break;
                                        }
                                    }
                                }
                                linehd.clear();
                            }
                            Err(err) => {  
                                self.msg_value = format!("error of {} reading {}", err, self.sortinname);
                                self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                bolok = false;
                                break;
                            }
                       };
                   }
                   if bolok {
                       let numlist = llist.len();
                       if numlist < 1 {
                           if bolend {
                               self.msg_value = "end of sorted input file and no more files".to_string();
                               self.mess_color = Color::from([0.0, 1.0, 0.0]);
                           } else {
                               self.msg_value = format!("error no files in list for {}", lrefname);
                               self.mess_color = Color::from([1.0, 0.0, 0.0]);
                           }
                       } else {
                           self.mess_color = Color::from([0.0, 1.0, 0.0]);
                           self.listindex = linenum;
                           self.listrefname = llist;
                           self.bkrefname = lrefname;
                           self.msg_value = format!("Please mount {} which has {} files", self.bkrefname, numlist);
                       }
                   }
               }
               Task::none()
            }
            Message::ExecPressed => {
               let (errcode, errstr) = execpress(self.sortinname.clone(), self.bkpath.clone(), self.targetdir.clone(), self.listrefname.clone());
               self.msg_value = errstr.to_string();
               if errcode == 0 {
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
                   Task::perform(Execx::execit(self.bkpath.clone(),self.targetdir.clone(), self.listrefname.clone(), self.tx_send.clone()), Message::ExecxFound)

               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
                   Task::none()
               }
            }
            Message::ExecxFound(Ok(exx)) => {
               self.msg_value = exx.errval.clone();
               self.mess_color = Color::from([1.0, 0.0, 0.0]);
               let mut n = 0;
               if exx.errcd == 0 {
                   for path in cd_drives() {
                        match Device::open(path.clone()){
                          Ok(drive) => {
                               n = n + 1;
                               match drive.eject() {
                                 Ok(()) => {
                                    self.mess_color = Color::from([0.0, 1.0, 0.0]);
                                 },
                                 Err(e) => {
                                    self.msg_value = format!("error {} in ejecting blu ray", e);
                                 }
                               }
                          }
                          Err(e) => {
                            self.msg_value = format!("error {} in opening blu ray", e);
                          }
                        }
                   }
                   if n < 1 {
                       self.msg_value = "no dvd drives found".to_string();
                   }
               }
               Task::none()
            }
            Message::ExecxFound(Err(_error)) => {
               self.msg_value = "error in copyx copyit routine".to_string();
               self.mess_color = Color::from([1.0, 0.0, 0.0]);
               Task::none()
            }
            Message::ProgressPressed => {
                   self.do_progress = true;
                   Task::perform(Progstart::pstart(), Message::ProgRtn)
            }
            Message::ProgRtn(Ok(_prx)) => {
              if self.do_progress {
                let mut inputval  = " ".to_string();
                let mut bgotmesg = false;
                let mut b100 = false;
                while let Ok(Some(input)) = self.rx_receive.try_next() {
                   inputval = input;
                   bgotmesg = true;
                }
                if bgotmesg {
                    let progvec: Vec<&str> = inputval[0..].split("|").collect();
                    let lenpg1 = progvec.len();
                    if lenpg1 == 4 {
                        let prog1 = progvec[0].to_string();
                        if prog1 == "Progress" {
                            let num_flt: f64 = progvec[1].parse().unwrap_or(-9999.0);
                            if num_flt < 0.0 {
                                println!("progress numeric not numeric: {}", inputval);
                            } else {
                                let dem_flt: f64 = progvec[2].parse().unwrap_or(-9999.0);
                                if dem_flt < 0.0 {
                                    println!("progress numeric not numeric: {}", inputval);
                                } else {
                                    self.progval = 100.0 * (num_flt / dem_flt);
                                    if dem_flt == num_flt {
                                        b100 = true;
                                    } else {
                                        self.msg_value = format!("md5sum progress: {:.3}gb of {:.3}gb {}", (num_flt/1000000000.0), (dem_flt/1000000000.0), progvec[3]);
                                        self.mess_color = Color::from([0.0, 0.0, 1.0]);
                                    }
                                }
                            }
                        } else {
                            println!("message not progress: {}", inputval);
                        }
                    } else {
                        println!("message not progress: {}", inputval);
                    }
                } 
                if b100 {
                    Task::none()   
                } else {         
                    Task::perform(Progstart::pstart(), Message::ProgRtn)
                }
              } else {
                Task::none()
              }
            }
            Message::ProgRtn(Err(_error)) => {
                self.msg_value = "error in Progstart::pstart routine".to_string();
                self.mess_color = Color::from([1.0, 0.0, 0.0]);
               Task::none()
            }

            Message::Size(size) => {
                self.screenwidth = size.width;
                self.screenheight = size.height;
                Task::none()
            }

        }
    }

    fn view(&self) -> Element<Message> {
        column![
            row![text("Message:").size(20),
                 text(&self.msg_value).size(20).color(*&self.mess_color),
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![button("get sorted input").on_press(Message::SortinPressed),
                 text("   sorted input:").size(20),text(&self.sortinname).size(20),
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![button("First Button").on_press(Message::FirstPressed),
                 button("Next Button").on_press(Message::NextPressed),
                 text("   current backup disk:").size(20),text(&self.bkrefname).size(20),
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![button("Search for bluray disc").on_press(Message::BkPressed),
                 text("   bluray label:").size(20),
                 text(&self.bklabel).size(20), text("       bluray mount:").size(20), text(&self.bkpath).size(20),
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![checkbox("Alternative label", self.alt_bool).on_toggle(Message::Alt).size(20),
                 text("        Alternative label name: ").size(20),
                 text_input("No input....", &self.altname)
                            .on_input(Message::AltnameChanged).padding(10).size(20),
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![button("Target directory Button").on_press(Message::TargetdirPressed),
                 text(&self.targetdir).size(20).width(1000)
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![Space::with_width(200),
                 button("Exec Button").on_press(Message::ExecPressed),
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![button("Start Progress Button").on_press(Message::ProgressPressed).height(50),
                 progress_bar(0.0..=100.0,self.progval as f32),
                 text(format!("{:.2}%", &self.progval)).size(30),
            ].align_y(Alignment::Center).spacing(5).padding(10),
            row![text("need to input sorted input from FindBackupDB").size(20),
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![text("sort --field-separator='|' --key=1 --output=just1nnsorted.neout < just1nn.neout").size(20),
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![text("get blu ray disk and get path and label").size(20),
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![text("execute will copy files into the target directory from the backup disk").size(20),
            ].align_y(Alignment::Center).spacing(10).padding(10),
            row![text(format!("width: {:.1}   height: {:.1}", &self.screenwidth, &self.screenheight)).size(20), 
            ].align_y(Alignment::Center).spacing(10).padding(10),

        ]
        .padding(5)
        .align_x(Alignment::Start)
        .into()
    }

    fn theme(&self) -> Theme {
       Theme::Dracula
    }
    
    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _window| match event {
            Event::Window(window::Event::Resized(size)) => {
                Some(Message::Size(size))
            }
            _ => None,
        })
    }
            

}

#[derive(Debug, Clone)]
struct Execx {
    errcd: u32,
    errval: String,
}

impl Execx {
//    const TOTAL: u16 = 807;

    async fn execit(bkpath: String, targetdir: String, listref: Vec<String>,  tx_send: mpsc::UnboundedSender<String>,) -> Result<Execx, Error> {
//     const BACKSPACE: char = 8u8 as char;
     let mut errstring  = "Completed blu ray listing".to_string();
     let mut errcode: u32 = 0;
     let mut numprocess = 0;
     let totallin = listref.len();
     let mut linenum = 0;
     let mut bolok = true;
     let start_time = timeInstant::now();

     for entryx in listref {
          linenum = linenum + 1;
          let vecline: Vec<&str> = entryx.split("|").collect();
          let bkdir = vecline[1].to_string();
          let rfilename = vecline[2].to_string();
          let rdir = vecline[7].to_string();
          let fullsource: String;
          let fulltarget: String;
          if bkdir.contains("\\") {
              let bkdirr = bkdir.replace("\\", "/");
              fullsource = format!("{}/{}/{}", bkpath, bkdirr, rfilename);
          } else {
              fullsource = format!("{}/{}/{}", bkpath, bkdir, rfilename);
          }
          if !Path::new(&fullsource).exists() {
              errstring = format!("********* Copy: ERROR {} does not exist **********", fullsource);
              bolok = false;
              break;
          }
          if rdir.contains("\\") {
              let rdirr = rdir.replace("\\", "/");
              fulltarget = format!("{}/{}/{}", targetdir, rdirr, rfilename);
          } else {
              fulltarget = format!("{}/{}/{}", targetdir, rdir, rfilename);
          }
          if Path::new(&fulltarget).exists() {
              errstring = format!("********* Copy: ERROR {} already exists **********", fulltarget);
              bolok = false;
              break;
          }
          if numprocess < 4 {
              stdCommand::new("cp")
                           .arg("-p")
                           .arg(&fullsource)
                           .arg(&fulltarget)
                           .spawn()
                           .expect("failed to execute process");
              numprocess = numprocess + 1;
          } else {
              let _output = stdCommand::new("cp")
                                         .arg("-p")
                                         .arg(&fullsource)
                                         .arg(&fulltarget)
                                         .output()
                                         .expect("failed to execute process");
              numprocess = 0;
          }
          let diffx = start_time.elapsed();
          let minsx: f64 = diffx.as_secs() as f64/60 as f64;
          let datexx = Local::now();
          let msgx = format!("Progress|{}|{}| elapsed time {:.1} mins at {} {} of {}", linenum, totallin, minsx, datexx.format("%H:%M:%S"), linenum, totallin);
          tx_send.unbounded_send(msgx).unwrap();
     }
     if bolok {
         errstring  = format!("Completed copying {} of {}", linenum, totallin);
         errcode = 0;
     }
     let msgx = format!("Progress|{}|{}| end of copy process", totallin, totallin);
     tx_send.unbounded_send(msgx).unwrap();
     Ok(Execx {
            errcd: errcode,
            errval: errstring,
        })
    }
}
#[derive(Debug, Clone)]
pub enum Error {
//    APIError,
//    LanguageError,
}

// loop thru by sleeping for 5 seconds
#[derive(Debug, Clone)]
pub struct Progstart {
//    errcolor: Color,
//    errval: String,
}

impl Progstart {

    pub async fn pstart() -> Result<Progstart, Error> {
//     let errstring  = " ".to_string();
//     let colorx = Color::from([0.0, 0.0, 0.0]);
     sleep(timeDuration::from_secs(5));
     Ok(Progstart {
//            errcolor: colorx,
//            errval: errstring,
        })
    }
}
