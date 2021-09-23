// 10001,"Avalon","Avalon"," 1","Freshwater",N,N,"",0,121,"Chapman","","Matthew",
// "Conservative","Conservateur",N,N,33

use serde::Deserialize;

/// A particular poll within a riding. We expect an entry per party.
#[derive(Deserialize)]
struct Poll {
    #[serde(rename = "Electoral District Number/Numéro de circonscription")]
    riding: String,
    #[serde(rename = "Political Affiliation Name_English")]
    party: Party,
    #[serde(rename = "Candidate’s Family Name/Nom de famille du candidat")]
    last_name: String,
    #[serde(rename = "Candidate’s First Name/Prénom du candidat")]
    first_name: String,
    #[serde(rename = "Candidate Poll Votes Count/Votes du candidat pour le bureau")]
    votes: u32,
}

/// A candidate's political party.
#[derive(Deserialize)]
enum Party {
    #[serde(rename = "Liberal")]
    LIB,
    #[serde(rename = "Conservative")]
    CON,
    #[serde(rename = "NDP-New Democratic Party")]
    NDP,
    #[serde(rename = "Bloc Québécois")]
    BLQ,
    #[serde(rename = "Green Party")]
    GRN,
    #[serde(rename = "People's Party")]
    PPC,
    #[serde(rename = "Independent")]
    IND,
    #[serde(other)]
    OTH,
}

fn main() {
    println!("Hello, world!");
}
