//! Data models

mod iso8601;
mod serde_metas;

use core::fmt;
use std::{cmp::Reverse, collections::HashMap, env, fs, hash::Hash};

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use time::{serde::timestamp, Duration, OffsetDateTime};
use tracing::info;

use crate::request::{igdb::IgdbRequestor, resource::ResourceRequestor};
pub use iso8601::Iso8601Date;

const LIST_FILENAME: &str = "list.json";
const META_FILENAME: &str = "meta.json";
const META_TEMPLATE_FILENAME: &str = "meta_template.json";
pub const LOGO_FILENAME: &str = "res/logo.png";

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct Lists(pub HashMap<Iso8601Date, List>);

impl Lists {
    fn latest(&self) -> Option<&List> {
        self.0.iter().max_by_key(|(k, _)| *k).map(|(_, v)| v)
    }
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct List(pub Vec<GameId>);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GameId {
    Igdb(u32),
    Other(String),
    None,
}

impl fmt::Display for GameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Igdb(id) => write!(f, "{id}"),
            Self::Other(id) => write!(f, "{id}"),
            Self::None => Ok(()),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
pub enum AgeRatingCategory {
    Esrb = 1,
    Pegi = 2,
    Cero = 3,
    Usk = 4,
    Grac = 5,
    ClassInd = 6,
    Acb = 7,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
pub enum AgeRatingRating {
    Three = 1,
    Seven = 2,
    Twelve = 3,
    Sixteen = 4,
    Eighteen = 5,
    Rp = 6,
    Ec = 7,
    E = 8,
    E10 = 9,
    T = 10,
    M = 11,
    Ao = 12,
    CeroA = 13,
    CeroB = 14,
    CeroC = 15,
    CeroD = 16,
    CeroZ = 17,
    Usk0 = 18,
    Usk6 = 19,
    Usk12 = 20,
    Usk16 = 21,
    Usk18 = 22,
    GracAll = 23,
    GracTwelve = 24,
    GracFifteen = 25,
    GracEighteen = 26,
    GracTesting = 27,
    ClassIndL = 28,
    ClassIndTen = 29,
    ClassIndTwelve = 30,
    ClassIndFourteen = 31,
    ClassIndSixteen = 32,
    ClassIndEighteen = 33,
    AcbG = 34,
    AcbPG = 35,
    AcbM = 36,
    AcbMa15 = 37,
    AcbR18 = 38,
    AcbRc = 39,
}

#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr)]
pub enum PlatformCategory {
    Console = 1,
    Arcade = 2,
    Platform = 3,
    OperatingSystem = 4,
    PortableConsole = 5,
    Computer = 6,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct AgeRating {
    pub category: AgeRatingCategory,
    pub rating: AgeRatingRating,
    pub rating_cover_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UrlField {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NameField {
    pub name: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct GameEngine {
    pub name: String,
    pub logo: Option<UrlField>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Company {
    pub country: Option<u32>,
    pub logo: Option<UrlField>,
    pub name: String,
    #[serde(default, with = "timestamp::option")]
    pub start_date: Option<OffsetDateTime>,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Serialize, Deserialize)]
pub struct InvolvedCompany {
    pub developer: bool,
    pub porting: bool,
    pub publisher: bool,
    pub supporting: bool,
    pub company: Company,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Serialize, Deserialize)]
pub struct MultiplayerMode {
    pub campaigncoop: bool,
    pub lancoop: bool,
    pub offlinecoop: bool,
    pub onlinecoop: bool,
}

#[allow(clippy::struct_field_names)]
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Platform {
    pub category: Option<PlatformCategory>,
    pub name: String,
    pub generation: Option<u32>,
    pub platform_logo: Option<UrlField>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct DateField {
    #[serde(default, with = "timestamp::option")]
    pub date: Option<OffsetDateTime>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub id: GameId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub age_ratings: Vec<AgeRating>,
    pub aggregated_rating: Option<f64>,
    pub aggregated_rating_count: Option<u32>,
    pub cover: Option<UrlField>,
    #[serde(with = "timestamp")]
    pub first_release_date: OffsetDateTime,
    pub franchise: Option<NameField>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub game_engines: Vec<GameEngine>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub game_modes: Vec<NameField>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genres: Vec<NameField>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub involved_companies: Vec<InvolvedCompany>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<NameField>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub multiplayer_modes: Vec<MultiplayerMode>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub platforms: Vec<Platform>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub player_perspectives: Vec<NameField>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub release_dates: Vec<DateField>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub themes: Vec<NameField>,
    pub rating: Option<f64>,
    pub rating_count: Option<u32>,
    pub total_rating: Option<f64>,
    pub total_rating_count: Option<u32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Metas(#[serde(with = "serde_metas")] pub HashMap<GameId, Meta>);

#[derive(Debug, Clone, Copy)]
pub enum RatingKind {
    User,
    Critic,
    Total,
}

impl fmt::Display for RatingKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::User => "IGDB User Ranking",
                Self::Critic => "IGDB Critic Ranking",
                Self::Total => "IGDB Ranking",
            }
        )
    }
}

#[derive(Debug)]
pub struct Data {
    pub lists: Lists,
    pub metas: Metas,
    pub res: ResourceRequestor,
}

impl Data {
    pub async fn new(client: Client) -> Result<Self> {
        info!("Loading lists");
        let lists: Lists = serde_json::from_str(&fs::read_to_string(LIST_FILENAME)?)?;
        info!("Loaded lists");
        info!("Loading metadata");
        let mut metas = if fs::exists(META_FILENAME)? {
            serde_json::from_str(&fs::read_to_string(META_FILENAME)?)?
        } else if fs::exists(META_TEMPLATE_FILENAME)? {
            fs::copy(META_TEMPLATE_FILENAME, META_FILENAME)?;
            serde_json::from_str(&fs::read_to_string(META_FILENAME)?)?
        } else {
            Metas::default()
        };

        let missing_metas = lists
            .latest()
            .ok_or_else(|| anyhow!("error"))?
            .0
            .iter()
            .filter_map(|id| {
                if metas.0.contains_key(id) {
                    None
                } else {
                    Some(if matches!(id, GameId::Igdb(_)) {
                        Ok(id.clone())
                    } else {
                        Err(anyhow!("Missing metadata for \"{}\"", id))
                    })
                }
            })
            .collect::<Result<Vec<_>>>()?;

        if !missing_metas.is_empty() {
            info!("Downloading missing metadata");
            let client_id = env::var("CLIENT_ID")?;
            let client_secret = env::var("CLIENT_SECRET")?;
            let mut req = IgdbRequestor::new(client.clone(), &client_id, &client_secret);
            metas.0.extend(req.games(missing_metas.as_slice()).await?.0);
            fs::write(META_FILENAME, &serde_json::to_string_pretty(&metas)?)?;
            info!("Downloaded missing metadata");
        }

        info!("Loaded metadata");

        Ok(Self {
            lists,
            metas,
            res: ResourceRequestor::new(client),
        })
    }

    /// All dates when list was changed
    pub fn dates(&self) -> Vec<Iso8601Date> {
        let mut dates = self.lists.0.keys().copied().collect::<Vec<_>>();
        dates.sort();
        dates
    }

    /// Time that each game spent on the top / bottom of the list
    pub fn extrema(&self, top: bool) -> Vec<(&GameId, Duration)> {
        let dates = self.dates();
        let mut extrema = HashMap::new();

        for period in dates.windows(2) {
            let list = &self.lists.0[&period[0]].0;
            let extremum = &list[if top { 0 } else { list.len() - 1 }];
            let duration = period[1].0 - period[0].0;
            extrema
                .entry(extremum)
                .and_modify(|e| *e += duration)
                .or_insert(duration);
        }

        let mut extrema = extrema.into_iter().collect::<Vec<_>>();
        extrema.sort_by_key(|top| Reverse(top.1));
        extrema
    }

    pub fn igdb_list(&self, kind: RatingKind) -> Vec<(f64, &Meta)> {
        let mut igdb_list = self
            .metas
            .0
            .values()
            .filter_map(|meta| {
                match kind {
                    RatingKind::User => meta.rating,
                    RatingKind::Critic => meta.aggregated_rating,
                    RatingKind::Total => meta.total_rating,
                }
                .map(|rating| (rating, meta))
            })
            .collect::<Vec<_>>();
        igdb_list.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        igdb_list
    }

    /// Most common elements from metas
    pub fn most_common<'a, FE, FH, I, T, N>(&'a self, extract: FE, hash: FH) -> Vec<(u32, &'a T)>
    where
        FE: Fn(&'a Meta) -> I,
        FH: Fn(&'a T) -> N,
        I: Iterator<Item = &'a T>,
        N: Hash + Eq + 'a,
        T: 'a,
    {
        let mut values = HashMap::<N, (u32, &T)>::new();
        for value in self.metas.0.values().flat_map(extract) {
            values
                .entry(hash(value))
                .and_modify(|e| e.0 += 1)
                .or_insert((1, value));
        }
        let mut values = values.values().copied().collect::<Vec<_>>();
        values.sort_by_key(|top| Reverse(top.0));
        values
    }

    /// Difference in list position between The List and the IGDB ranking
    pub fn igdb_diffs(&self) -> Option<Vec<(i32, &Meta)>> {
        let igdb_list = self.igdb_list(RatingKind::Total);
        let latest_list = self.lists.latest()?;
        let mut diffs = igdb_list
            .iter()
            .enumerate()
            .map(|(i, (_, meta))| {
                latest_list
                    .0
                    .iter()
                    .position(|id| *id == meta.id)
                    .map(|id| (id as i32 - i as i32, *meta))
            })
            .collect::<Option<Vec<_>>>()?;
        diffs.sort_by_key(|x| x.0);
        Some(diffs)
    }

    pub fn latest(&self) -> Option<&List> {
        self.lists.latest()
    }

    pub fn release_date_range(&self) -> Option<(OffsetDateTime, OffsetDateTime)> {
        Some((
            self.metas
                .0
                .values()
                .min_by_key(|meta| meta.first_release_date)?
                .first_release_date,
            self.metas
                .0
                .values()
                .max_by_key(|meta| meta.first_release_date)?
                .first_release_date,
        ))
    }
}
