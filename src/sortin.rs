use rfd::FileDialog;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command as stdCommand;

pub fn sortin (inputval: String) -> (u32, String, String, String, u64, Vec<String>) {
     let mut errcode: u32;
     let mut errstring: String;
     let mut new_input: String;
     let mut linenum: u64 = 0;
     let mut lrefname = "---".to_string();
     let mut llist: Vec<String> = Vec::new();
     if Path::new(&inputval).exists() {
         let getpath = PathBuf::from(&inputval);
         let getdir = getpath.parent().unwrap();
         new_input = getdir.to_str().unwrap().to_string();
     } else {
         new_input = "/".to_string();
     }
     let newfile = FileDialog::new()
         .set_directory(&new_input)
         .pick_file();
     if newfile == None {
         errstring = "error getting file -- possible cancel key hit".to_string();
         errcode = 1;
     } else {
         new_input = newfile.as_ref().expect("REASON").display().to_string();
         let outputx = stdCommand::new("wc")
                         .arg("-l")
                         .arg(&new_input)
                         .output()
                         .expect("failed to execute process");
         let stroutx = String::from_utf8_lossy(&outputx.stdout);
         let vecout: Vec<&str> = stroutx.split(" ").collect();
         let numlinesx: i64 = vecout[0].parse().unwrap_or(-9999);
         if numlinesx == -9999 {
             errstring = format!("size of {} is invalid for wc -l command call", vecout[0]);
             errcode = 2;
         } else {
             let rows_num = numlinesx as u64;
             if rows_num < 2 {
                 errstring = format!("size of {} is less than 2 for {}", rows_num, new_input);
                 errcode = 3;
             } else {
                 errstring = "got file".to_string();
                 errcode = 0;
                 let file = File::open(new_input.clone()).unwrap();
                 let mut reader = BufReader::new(file);
                 let mut linehd = String::new();
                 loop {
                       match reader.read_line(&mut linehd) {
                            Ok(bytes_read) => {
                                if bytes_read == 0 {
                                    if linenum < 1 {
                                         errstring = format!("error bytes_read == 0 for {}", new_input);
                                         errcode = 4;
                                    }
                                    break;
                                }
                                linenum = linenum + 1;
                                if linenum < 2 {
                                    let cnt = linehd.matches("|").count();
                                    if cnt < 7 {
                                        errstring = format!("first line of sorted input file is not valid: {}", linehd);
                                        errcode = 5;
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
                                errstring = format!("error of {} reading {}", err, new_input);
                                errcode = 6;
                                break;
                            }
                       };
                 }
             }
         }
     } 
     (errcode, errstring, new_input, lrefname, linenum, llist)
}

