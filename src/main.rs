use clap::{crate_version, ArgEnum, Clap};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Not;

#[derive(Clap)]
#[clap(author = "Colin Woodbury", version = crate_version!(), about = "Canadian Federal Election data")]
struct Args {
    /// Total votes and seats for every party.
    #[clap(group = "choice", long, display_order = 1)]
    total: bool,

    /// Ridings where CON would have won if all PPC had voted CON.
    #[clap(group = "choice", long, display_order = 1)]
    conppc: bool,

    /// Ridings ordered by margin of victory.
    #[clap(group = "choice", long, display_order = 1)]
    margins: bool,

    /// How a given Party did in every riding.
    #[clap(group = "choice", long, display_order = 1, arg_enum)]
    party: Option<Party>,

    /// The election year to consider.
    #[clap(long, display_order = 2, possible_values = &["2008", "2011", "2015", "2019"], default_value = "2019")]
    year: usize,
}

#[derive(Debug)]
struct Riding {
    name: String,
    candidates: HashMap<Party, Candidate>,
}

impl Riding {
    /// Was the given [`Party`] the winner of this riding?
    fn was_winner(&self, party: &Party) -> bool {
        party == &self.winner()
    }

    /// The victories party in this riding.
    fn winner(&self) -> Party {
        self.candidates
            .iter()
            .max_by(|(_, a), (_, b)| a.votes.cmp(&b.votes))
            .unwrap()
            .0
            .clone()
    }

    /// The margin of victory for this `Riding`.
    fn victory_margin(&self) -> f32 {
        let mut votes: Vec<_> = self.candidates.values().map(|c| c.votes).collect();
        votes.sort_by(|a, b| b.cmp(&a));
        let total_votes: usize = votes.iter().sum();
        let winner = votes[0] as f32;
        let second = votes[1] as f32;

        (winner - second) / total_votes as f32
    }

    /// The total votes in this `Riding`.
    fn total_votes(&self) -> usize {
        self.candidates.values().map(|c| c.votes).sum()
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
    #[serde(alias = "Candidate's Family Name/Nom de famille du candidat")]
    last_name: String,
    #[serde(rename = "Candidate’s First Name/Prénom du candidat")]
    #[serde(alias = "Candidate's First Name/Prénom du candidat")]
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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone, ArgEnum)]
enum Party {
    #[serde(rename = "Liberal")]
    LIB,
    #[serde(rename = "Conservative")]
    CON,
    #[serde(rename(
        deserialize = "NDP-New Democratic Party",
        serialize = "New Democratic Party"
    ))]
    NDP,
    #[serde(rename = "Bloc Québécois")]
    BLQ,
    #[serde(rename = "Green Party")]
    GRN,
    #[serde(rename = "People's Party")]
    PPC,
    #[serde(rename = "Independent", alias = "No Affiliation")]
    IND,
    // --- Small parties --- //
    #[serde(rename = "Libertarian")]
    LTN,
    #[serde(
        rename(deserialize = "Parti Rhinocéros Party", serialize = "Rhinoceros Party"),
        alias = "Rhinoceros",
        alias = "neorhino.ca"
    )]
    RIN,
    #[serde(rename = "National Citizens Alliance")]
    NCA,
    #[serde(rename = "Animal Protection Party")]
    APP,
    #[serde(
        rename = "Animal Alliance/Environment Voters",
        alias = "AAEV Party of Canada"
    )]
    AAE,
    #[serde(rename = "Democratic Advancement")]
    DAD,
    #[serde(rename(serialize = "Alliance of the North"))]
    ATN,
    #[serde(rename(
        deserialize = "Forces et Démocratie - Allier les forces de nos régions",
        serialize = "Forces et Démocratie"
    ))]
    FED,
    #[serde(rename(deserialize = "VCP", serialize = "Veteran's Coalition"))]
    VCP,
    #[serde(rename = "Christian Heritage Party", alias = "CHP Canada")]
    CHP,
    #[serde(rename = "Pour l'Indépendance du Québec")]
    PIQ,
    #[serde(rename = "Communist")]
    COM,
    /// Marxist-Leninist
    #[serde(
        rename(deserialize = "ML", serialize = "Marxist-Leninist"),
        alias = "Marxist-Leninist"
    )]
    MXL,
    #[serde(
        rename(deserialize = "UPC", serialize = "United Party of Canada"),
        alias = "United Party"
    )]
    UPC,
    #[serde(
        rename(deserialize = "Pirate", serialize = "Pirate Party"),
        alias = "Pirate Party"
    )]
    PIR,
    #[serde(rename = "Radical Marijuana")]
    RMJ,
    #[serde(rename(deserialize = "PC Party", serialize = "Progressive Canadian Party"))]
    PCP,
    #[serde(rename = "Stop Climate Change")]
    SCC,
    #[serde(rename(
        deserialize = "CFF - Canada's Fourth Front",
        serialize = "Canada's Fourth Front"
    ))]
    CFF,
    #[serde(rename = "Nationalist")]
    NAT,
    #[serde(rename = "Seniors Party")]
    SNR,
    #[serde(rename = "Canada Party")]
    CAD,
    #[serde(rename(serialize = "Canadian Action Party"))]
    CAP,
    #[serde(rename = "The Bridge")]
    TBR,
    PACT,
    #[serde(rename(serialize = "Western Block Party"))]
    WBP,
    #[serde(rename(serialize = "First Peoples National Party"))]
    FPNP,
    #[serde(rename = "Work Less Party")]
    WLP,
    #[serde(rename(serialize = "People's Political Power"))]
    PPP,
    #[serde(rename(
        deserialize = "NL First Party",
        serialize = "Newfoundland and Labrador First"
    ))]
    NLF,
}

#[derive(Serialize)]
struct VoteCount {
    party: Party,
    votes: usize,
    ratio: f32,
    seats: usize,
}

#[derive(Serialize)]
struct ComboVictory {
    riding: String,
    winner: Party,
    winner_votes: usize,
    con_ppc_votes: usize,
    difference: usize,
}

#[derive(Serialize)]
struct VictoryMargin {
    riding: String,
    winner: Party,
    margin: f32,
}

#[derive(Serialize)]
struct PartyResults {
    riding: String,
    party: Party,
    last_name: String,
    first_name: String,
    votes: usize,
    ratio: f32,
    won: bool,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    let data = format!("data/{}", args.year);

    let mut polls: Vec<Poll> = std::fs::read_dir(data)?
        .filter_map(|de| de.ok())
        .filter_map(|de| csv::Reader::from_path(de.path()).ok())
        // Unfortunate `collect` due to the `reader` being owned.
        .flat_map(|mut reader| reader.deserialize::<Poll>().collect::<Vec<_>>().into_iter())
        .collect::<Result<Vec<Poll>, _>>()?;

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
    } else if args.margins {
        victory_margins(unified);
    } else if let Some(party) = args.party {
        party_results(unified, party);
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

/// How a given [`Party`] did in every riding.
fn party_results(polls: Vec<Poll>, party: Party) {
    let mut results: Vec<_> = ridings(polls)
        .into_iter()
        .filter_map(|mut riding| {
            let won = riding.was_winner(&party);
            riding.candidates.remove(&party).map(|c| (riding, won, c))
        })
        .map(|(riding, won, c)| {
            //
            let total = riding.total_votes() + c.votes;
            let ratio = c.votes as f32 / total as f32;

            PartyResults {
                riding: riding.name,
                party: party.clone(),
                last_name: c.last_name,
                first_name: c.first_name,
                votes: c.votes,
                ratio,
                won,
            }
        })
        .collect();

    results.sort_by(|a, b| a.ratio.partial_cmp(&b.ratio).unwrap_or(Ordering::Less));

    println!("{}", serde_json::to_string(&results).unwrap());
}

/// Ordered list of ridings by the victory margin.
fn victory_margins(polls: Vec<Poll>) {
    let mut margins: Vec<_> = ridings(polls)
        .into_iter()
        .map(|riding| {
            let margin = riding.victory_margin();
            let winner = riding.winner();

            VictoryMargin {
                winner: winner.clone(),
                riding: riding.name,
                margin,
            }
        })
        .collect();

    margins.sort_by(|a, b| a.margin.partial_cmp(&b.margin).unwrap_or(Ordering::Less));

    println!("{}", serde_json::to_string(&margins).unwrap());
}

/// For ridings in which the Conservatives lost, would the combined CON + PPC
/// have swung the result?
///
/// False Assumption #1: All PPC voters are naturally right-wing and would have
/// otherwise voted CON. Similar to Trump voters in the USA, a section of the
/// voter base are those disenfranchised with the existing parties and who just
/// want a new alternative. While right-wing in nature, surely the PPC are
/// drawing voters from all parts of Canada.
///
/// False Assumption #2: Everyone has a fixed party loyalty, and nobody ever
/// votes for other reasons. In reality there are a myriad of reasons why people
/// choose a particular party to vote for in a particular riding in a particular
/// year.
fn ppc_con(polls: Vec<Poll>) {
    let wins: Vec<_> = ridings(polls)
        .iter()
        .filter(|riding| riding.was_winner(&Party::CON).not())
        .filter_map(|riding| {
            let cs = &riding.candidates;
            let winner = riding.winner();
            cs.get(&winner).and_then(|win| {
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

/// Vote and seat totals per party.
fn totals(unified: Vec<Poll>) {
    let mut votes: HashMap<Party, usize> = HashMap::new();
    let mut seats: HashMap<Party, usize> = HashMap::new();

    for riding in ridings(unified) {
        let party = riding.winner();
        let entry = seats.entry(party).or_insert(0);
        *entry += 1;

        for (party, candidate) in riding.candidates {
            let entry = votes.entry(party).or_insert(0);
            *entry += candidate.votes;
        }
    }

    let total_votes: usize = votes.values().sum();
    let vote_counts: Vec<VoteCount> = votes
        .into_iter()
        .map(|(party, votes)| VoteCount {
            seats: seats.remove(&party).unwrap_or(0),
            party,
            votes,
            ratio: votes as f32 / total_votes as f32,
        })
        .collect();

    println!("{}", serde_json::to_string(&vote_counts).unwrap());
}
