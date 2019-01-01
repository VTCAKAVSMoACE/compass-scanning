extern crate reqwest;
extern crate select;

use select::document::Document;
use select::predicate::Name;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;

fn main() {
    let client = reqwest::Client::builder().timeout(Duration::from_secs(60)).build().unwrap();
    let first_page = client.get("https://compass-ssb.tamu.edu/pls/PROD/bwckschd.p_disp_dyn_sched").send().unwrap();
    assert!(first_page.status().is_success());
    if let Some(opt) = Document::from_read(first_page).unwrap()
        .find(Name("option"))
        .filter_map(|n| n.attr("value"))
        .nth(1) {
            let resp = client
                .post("https://compass-ssb.tamu.edu/pls/PROD/bwckgens.p_proc_term_date")
                .form(&[
                    ("p_term", opt),
                    ("p_calling_proc", "bwckschd.p_disp_dyn_sched"),
                ])
                .send()
                .unwrap();
            assert!(resp.status().is_success());
            Document::from_read(resp).unwrap()
                .find(Name("select"))
                .next()
                .unwrap()
                .find(Name("option"))
                .filter_map(|n| n.attr("value"))
                .for_each(|val| {
                    let mut results = client
                        .post("https://compass-ssb.tamu.edu/pls/PROD/bwckschd.p_get_crse_unsec")
                        .form(&[
                            ("term_in", opt),
                            ("sel_subj", "dummy"),
                            ("sel_day", "dummy"),
                            ("sel_schd", "dummy"),
                            ("sel_insm", "dummy"),
                            ("sel_camp", "dummy"),
                            ("sel_levl", "dummy"),
                            ("sel_sess", "dummy"),
                            ("sel_instr", "dummy"),
                            ("sel_ptrm", "dummy"),
                            ("sel_attr", "dummy"),
                            ("sel_subj", val),
                            ("sel_crse", ""),
                            ("sel_title", ""),
                            ("sel_schd", "%"),
                            ("sel_insm", "%"),
                            ("sel_from_cred", ""),
                            ("sel_to_cred", ""),
                            ("sel_camp", "%"),
                            ("sel_levl", "%"),
                            ("sel_ptrm", "%"),
                            ("sel_instr", "%"),
                            ("sel_attr", "%"),
                            ("begin_hh", "0"),
                            ("begin_mi", "0"),
                            ("begin_ap", "a"),
                            ("end_hh", "0"),
                            ("end_mi", "0"),
                            ("end_ap", "a"),
                        ])
                        .send()
                        .unwrap();
                    assert!(results.status().is_success());
                    let mut file = File::create(format!("{}.html", val)).unwrap();
                    file.write_all(results.text().unwrap().as_bytes()).unwrap();
                    println!("Scanned and saved {}.", val);
                    thread::sleep(Duration::from_secs(30));
                });
    }
}
