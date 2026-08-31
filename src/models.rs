use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Sitzung {
    pub id: String,
    pub datetime: String,
    pub ort: String,
    pub typ: String,
    pub antragsfrist: String,
    pub legislatur_periode: LegislaturPeriode,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct LegislaturPeriode {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Role {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Antrag {
    pub id: String,
    pub titel: String,
    pub antragstext: String,
    pub begruendung: String,
    pub erstellt_am: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct TopWithAntraege {
    pub id: String,
    pub weight: i64,
    pub name: String,
    pub inhalt: String,
    pub typ: String,
    #[serde(default)]
    pub antraege: Vec<Antrag>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateSitzung {
    pub datetime: String,
    pub ort: String,
    pub typ: String,
    pub antragsfrist: String,
    pub legislative_period: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateAntrag {
    pub titel: String,
    pub antragstext: String,
    pub begruendung: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateTop {
    pub name: String,
    pub typ: String,
    pub inhalt: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct AssocAntrag {
    pub antrag_id: String,
}
