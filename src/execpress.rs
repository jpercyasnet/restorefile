use std::path::Path;

pub fn execpress (sortin: String, bkdir: String, targetdir: String, list: Vec<String>) -> (u32, String) {
     let mut errcode: u32 = 0;
     let mut errstring: String = "all good and now process execution".to_string();
     if Path::new(&sortin).exists() {
         if Path::new(&targetdir).exists() {
             if Path::new(&bkdir).exists() {
                 if list.len() < 1 {
                     errstring = "the list of files is less than 1".to_string();
                     errcode = 1;
                 }
             } else {
                 errstring = "the blu ray disk directory does not exist".to_string();
                 errcode = 2;
             }
         } else {
             errstring = "the target directory does not exist".to_string();
             errcode = 3;
         }
     } else {
         errstring = "the sorted input does not exist".to_string();
         errcode = 4;
     }
     (errcode, errstring)
}
