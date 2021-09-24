use clap::{crate_version, Clap};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Not;

#[derive(Clap)]
#[clap(author = "Colin Woodbury", version = crate_version!(), about = "Canadian Federal Election data")]
struct Args {
    /// Total votes for every party.
    #[clap(group = "choice", long, display_order = 1)]
    total: bool,

    /// Ridings where CON would have won if there was no PPC split.
    #[clap(group = "choice", long, display_order = 1)]
    conppc: bool,
}

#[derive(Debug)]
struct Riding {
    name: String,
    candidates: HashMap<Party, Candidate>,
}

impl Riding {
    /// Was the given [`Party`] the winner of this riding?
    fn was_winner(&self, party: Party) -> bool {
        &party == self.winner()
    }

    /// The victories party in this riding.
    fn winner(&self) -> &Party {
        self.candidates
            .iter()
            .max_by(|(_, a), (_, b)| a.votes.cmp(&b.votes))
            .unwrap()
            .0
    }
}

#[derive(Debug)]
struct Candidate {
    last_name: String,
    first_name: String,
    votes: usize,
}

/// A particular poll within a riding. We expect an entry per party.
#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
struct Poll {
    #[serde(rename = "Electoral District Name_English/Nom de circonscription_Anglais")]
    riding: String,
    #[serde(rename = "Political Affiliation Name_English/Appartenance politique_Anglais")]
    party: Party,
    #[serde(rename = "Candidate’s Family Name/Nom de famille du candidat")]
    last_name: String,
    #[serde(rename = "Candidate’s First Name/Prénom du candidat")]
    first_name: String,
    #[serde(rename = "Candidate Poll Votes Count/Votes du candidat pour le bureau")]
    votes: usize,
}

impl Poll {
    /// Fuse two polls from the (hopefully) same riding.
    fn fuse(mut self, other: Poll) -> Poll {
        self.votes += other.votes;
        self
    }
}

impl PartialOrd for Poll {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Poll {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.riding.cmp(&other.riding) {
            Ordering::Equal => match self.party.cmp(&other.party) {
                Ordering::Equal => self.last_name.cmp(&other.last_name),
                o => o,
            },
            o => o,
        }
    }
}

/// A candidate's political party.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone)]
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
    #[serde(rename = "Libertarian")]
    LTN,
    #[serde(rename = "Communist")]
    COM,
    #[serde(rename = "Independent")]
    IND,
    // --- Small parties --- //
    #[serde(rename = "Parti Rhinocéros Party")]
    RIN,
    #[serde(rename = "National Citizens Alliance")]
    NCA,
    #[serde(rename = "Animal Protection Party")]
    APP,
    #[serde(rename = "VCP")]
    VCP,
    #[serde(rename = "Christian Heritage Party")]
    CHP,
    #[serde(rename = "Pour l'Indépendance du Québec")]
    PIQ,
    /// Marxist-Leninist
    #[serde(rename = "ML")]
    MXL,
    #[serde(rename = "No Affiliation")]
    NOA,
    #[serde(rename = "UPC")]
    UPC,
    #[serde(rename = "Radical Marijuana")]
    RMJ,
    #[serde(rename = "PC Party")]
    PCP,
    #[serde(rename = "Stop Climate Change")]
    SCC,
    #[serde(rename = "CFF - Canada's Fourth Front")]
    CFF,
    #[serde(rename = "Nationalist")]
    NAT,
}

#[derive(Serialize)]
struct VoteCount {
    party: Party,
    votes: usize,
    ratio: f32,
}

#[derive(Serialize)]
struct ComboVictory {
    riding: String,
    winner: Party,
    winner_votes: usize,
    con_ppc_votes: usize,
    difference: usize,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let mut polls: Vec<Poll> = std::fs::read_dir("data/2019")?
        .filter_map(|de| de.ok())
        .filter_map(|de| csv::Reader::from_path(de.path()).ok())
        // Unfortunate `collect` due to the `reader` being owned.
        .flat_map(|mut reader| reader.deserialize::<Poll>().collect::<Vec<_>>().into_iter())
        .filter_map(|poll| match poll {
            Err(e) => {
                eprintln!("{}", e);
                None
            }
            Ok(p) => Some(p),
        })
        .collect();

    // Sort by riding, then party.
    polls.sort();

    let unified: Vec<Poll> = polls
        .into_iter()
        // `clone` of enums is cheap.
        .group_by(|poll| poll.party.clone())
        .into_iter()
        .filter_map(|(_, group)| group.reduce(|a, b| a.fuse(b)))
        .collect();

    if args.total {
        totals(unified);
    } else if args.conppc {
        ppc_con(unified);
    }

    Ok(())
}

fn ridings(polls: Vec<Poll>) -> Vec<Riding> {
    polls
        .into_iter()
        .group_by(|poll| poll.riding.clone())
        .into_iter()
        .map(|(name, group)| {
            let candidates = group
                .map(|poll| {
                    let p = poll.party;
                    let c = Candidate {
                        last_name: poll.last_name,
                        first_name: poll.first_name,
                        votes: poll.votes,
                    };

                    (p, c)
                })
                .collect();

            Riding { name, candidates }
        })
        .collect()
}

/// For ridings in which the Conservatives lost, would the combined CON + PPC
/// have swung the result?
fn ppc_con(polls: Vec<Poll>) {
    let wins: Vec<_> = ridings(polls)
        .iter()
        .filter(|riding| riding.was_winner(Party::CON).not())
        .filter_map(|riding| {
            let cs = &riding.candidates;
            let winner = riding.winner();
            cs.get(winner).and_then(|win| {
                cs.get(&Party::CON).and_then(|con| {
                    cs.get(&Party::PPC)
                        .map(|ppc| (riding, winner, win, con, ppc))
                })
            })
        })
        .filter(|(_, _, w, c, p)| c.votes + p.votes > w.votes)
        .map(|(riding, wp, w, c, p)| ComboVictory {
            riding: riding.name.clone(),
            winner: wp.clone(),
            winner_votes: w.votes,
            con_ppc_votes: c.votes + p.votes,
            difference: (c.votes + p.votes) - w.votes,
        })
        .collect();

    println!("{}", serde_json::to_string(&wins).unwrap());
}

/// Vote totals per party.
fn totals(unified: Vec<Poll>) {
    let mut totals = HashMap::new();

    for poll in unified.iter() {
        let entry = totals.entry(&poll.party).or_insert(0);
        *entry += poll.votes;
    }

    let total_votes: usize = totals.values().sum();

    let vote_counts: Vec<VoteCount> = totals
        .into_iter()
        .map(|(party, votes)| VoteCount {
            party: party.clone(),
            votes,
            ratio: votes as f32 / total_votes as f32,
        })
        .collect();

    println!("{}", serde_json::to_string(&vote_counts).unwrap());
}
