use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug)]
struct Riding {
    name: String,
    candidates: HashMap<Party, Candidate>,
}

impl Riding {
    /// Was the given [`Party`] the winner of this riding?
    fn winner(&self, party: Party) -> bool {
        let winning_party: &Party = self
            .candidates
            .iter()
            .max_by(|(_, a), (_, b)| a.votes.cmp(&b.votes))
            .unwrap()
            .0;

        &party == winning_party
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
    con_ppc: usize,
    liberal: usize,
    difference: usize,
}

fn main() -> Result<(), std::io::Error> {
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

    totals(unified);
    // ppc_con(unified);

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

/// For ridings in which the Liberals won, would the combined CON + PPC have
/// swung the result?
fn ppc_con(polls: Vec<Poll>) {
    let wins: Vec<_> = ridings(polls)
        .iter()
        .filter(|riding| riding.winner(Party::LIB))
        .filter_map(|riding| {
            let cs = &riding.candidates;
            cs.get(&Party::LIB).and_then(|lib| {
                cs.get(&Party::CON)
                    .and_then(|con| cs.get(&Party::PPC).map(|ppc| (riding, lib, con, ppc)))
            })
        })
        .filter(|(_, l, c, p)| c.votes + p.votes > l.votes)
        .map(|(riding, l, c, p)| ComboVictory {
            riding: riding.name.clone(),
            con_ppc: c.votes + p.votes,
            liberal: l.votes,
            difference: (c.votes + p.votes) - l.votes,
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
