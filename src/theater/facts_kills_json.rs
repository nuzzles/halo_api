//! Complete killsource JSON cache projection. This preserves both native truths.
use super::facts_json_read::{Cell, FactsJsonDecode, FactsJsonState, JsonPointer, Shape};
use super::facts_json_write::{FactsJsonValue, FactsJsonWriter};
use super::*;
fn raw_string(cell: &Cell) -> Vec<u8> {
    let Cell::String(v) = cell else {
        unreachable!()
    };
    v.as_bytes().to_vec()
}
fn raw_strings(cell: &Cell) -> Option<Vec<Vec<u8>>> {
    let Cell::Sequence {
        backing,
        len,
        present,
    } = cell
    else {
        unreachable!()
    };
    present.then(|| backing[..*len].iter().map(raw_string).collect())
}
fn write_strings(value: &Option<Vec<Vec<u8>>>, w: &mut FactsJsonWriter) {
    if let Some(values) = value {
        w.raw(b"[");
        for (i, v) in values.iter().enumerate() {
            if i > 0 {
                w.raw(b",");
            }
            w.string(v);
        }
        w.raw(b"]");
    } else {
        w.raw(b"null");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsKillSourceHealth {
    pub film: Vec<u8>,
    pub candidates: i64,
    pub published: i64,
    pub unexplained_pair: i64,
    pub unexplained_self: i64,
    pub unexplained_bot_idx: i64,
    pub out_of_roster: i64,
    pub tag_out_of_catalogue_walk: i64,
    pub tag_out_of_catalogue_scan: i64,
    pub deaths_real: i64,
    pub deaths_covered: i64,
}
impl FactsJsonDecode for FactsKillsKillSourceHealth {
    const SHAPE: Shape = Shape::Struct {
        name: "grammar.KillSourceHealth",
        fields: &[
            ("Film", &Shape::String("string")),
            ("Candidates", &Shape::Signed(64, "int")),
            ("Published", &Shape::Signed(64, "int")),
            ("UnexplainedPair", &Shape::Signed(64, "int")),
            ("UnexplainedSelf", &Shape::Signed(64, "int")),
            ("UnexplainedBotIdx", &Shape::Signed(64, "int")),
            ("OutOfRoster", &Shape::Signed(64, "int")),
            ("TagOutOfCatalogueWalk", &Shape::Signed(64, "int")),
            ("TagOutOfCatalogueScan", &Shape::Signed(64, "int")),
            ("DeathsReal", &Shape::Signed(64, "int")),
            ("DeathsCovered", &Shape::Signed(64, "int")),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            film: raw_string(cell.field("Film")),
            candidates: FactsJsonDecode::from_cell(cell.field("Candidates")),
            published: FactsJsonDecode::from_cell(cell.field("Published")),
            unexplained_pair: FactsJsonDecode::from_cell(cell.field("UnexplainedPair")),
            unexplained_self: FactsJsonDecode::from_cell(cell.field("UnexplainedSelf")),
            unexplained_bot_idx: FactsJsonDecode::from_cell(cell.field("UnexplainedBotIdx")),
            out_of_roster: FactsJsonDecode::from_cell(cell.field("OutOfRoster")),
            tag_out_of_catalogue_walk: FactsJsonDecode::from_cell(
                cell.field("TagOutOfCatalogueWalk"),
            ),
            tag_out_of_catalogue_scan: FactsJsonDecode::from_cell(
                cell.field("TagOutOfCatalogueScan"),
            ),
            deaths_real: FactsJsonDecode::from_cell(cell.field("DeathsReal")),
            deaths_covered: FactsJsonDecode::from_cell(cell.field("DeathsCovered")),
        }
    }
}
impl FactsJsonValue for FactsKillsKillSourceHealth {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Film\":");
        w.string(&self.film);
        w.raw(b",\"Candidates\":");
        self.candidates.write_json(w);
        w.raw(b",\"Published\":");
        self.published.write_json(w);
        w.raw(b",\"UnexplainedPair\":");
        self.unexplained_pair.write_json(w);
        w.raw(b",\"UnexplainedSelf\":");
        self.unexplained_self.write_json(w);
        w.raw(b",\"UnexplainedBotIdx\":");
        self.unexplained_bot_idx.write_json(w);
        w.raw(b",\"OutOfRoster\":");
        self.out_of_roster.write_json(w);
        w.raw(b",\"TagOutOfCatalogueWalk\":");
        self.tag_out_of_catalogue_walk.write_json(w);
        w.raw(b",\"TagOutOfCatalogueScan\":");
        self.tag_out_of_catalogue_scan.write_json(w);
        w.raw(b",\"DeathsReal\":");
        self.deaths_real.write_json(w);
        w.raw(b",\"DeathsCovered\":");
        self.deaths_covered.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsAssistStats {
    pub kill_events: i64,
    pub attached: i64,
    pub multi: i64,
    pub named: i64,
    pub no_assist: i64,
    pub rejected_self: i64,
    pub rejected_victim: i64,
    pub rejected_roster: i64,
    pub flag_set: i64,
    pub assist_multi: i64,
    pub assist_extra_total: i64,
    pub assist_field_disagree: i64,
    pub killer_pct_over100: i64,
    pub assist_pct_over100: i64,
    pub par_la_fenetre: i64,
    pub gate15: bool,
}
impl FactsJsonDecode for FactsKillsAssistStats {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.AssistStats",
        fields: &[
            ("KillEvents", &Shape::Signed(64, "int")),
            ("Attached", &Shape::Signed(64, "int")),
            ("Multi", &Shape::Signed(64, "int")),
            ("Named", &Shape::Signed(64, "int")),
            ("NoAssist", &Shape::Signed(64, "int")),
            ("RejectedSelf", &Shape::Signed(64, "int")),
            ("RejectedVictim", &Shape::Signed(64, "int")),
            ("RejectedRoster", &Shape::Signed(64, "int")),
            ("FlagSet", &Shape::Signed(64, "int")),
            ("AssistMulti", &Shape::Signed(64, "int")),
            ("AssistExtraTotal", &Shape::Signed(64, "int")),
            ("AssistFieldDisagree", &Shape::Signed(64, "int")),
            ("KillerPctOver100", &Shape::Signed(64, "int")),
            ("AssistPctOver100", &Shape::Signed(64, "int")),
            ("ParLaFenetre", &Shape::Signed(64, "int")),
            ("Gate15", &<bool as FactsJsonDecode>::SHAPE),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            kill_events: FactsJsonDecode::from_cell(cell.field("KillEvents")),
            attached: FactsJsonDecode::from_cell(cell.field("Attached")),
            multi: FactsJsonDecode::from_cell(cell.field("Multi")),
            named: FactsJsonDecode::from_cell(cell.field("Named")),
            no_assist: FactsJsonDecode::from_cell(cell.field("NoAssist")),
            rejected_self: FactsJsonDecode::from_cell(cell.field("RejectedSelf")),
            rejected_victim: FactsJsonDecode::from_cell(cell.field("RejectedVictim")),
            rejected_roster: FactsJsonDecode::from_cell(cell.field("RejectedRoster")),
            flag_set: FactsJsonDecode::from_cell(cell.field("FlagSet")),
            assist_multi: FactsJsonDecode::from_cell(cell.field("AssistMulti")),
            assist_extra_total: FactsJsonDecode::from_cell(cell.field("AssistExtraTotal")),
            assist_field_disagree: FactsJsonDecode::from_cell(cell.field("AssistFieldDisagree")),
            killer_pct_over100: FactsJsonDecode::from_cell(cell.field("KillerPctOver100")),
            assist_pct_over100: FactsJsonDecode::from_cell(cell.field("AssistPctOver100")),
            par_la_fenetre: FactsJsonDecode::from_cell(cell.field("ParLaFenetre")),
            gate15: FactsJsonDecode::from_cell(cell.field("Gate15")),
        }
    }
}
impl FactsJsonValue for FactsKillsAssistStats {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"KillEvents\":");
        self.kill_events.write_json(w);
        w.raw(b",\"Attached\":");
        self.attached.write_json(w);
        w.raw(b",\"Multi\":");
        self.multi.write_json(w);
        w.raw(b",\"Named\":");
        self.named.write_json(w);
        w.raw(b",\"NoAssist\":");
        self.no_assist.write_json(w);
        w.raw(b",\"RejectedSelf\":");
        self.rejected_self.write_json(w);
        w.raw(b",\"RejectedVictim\":");
        self.rejected_victim.write_json(w);
        w.raw(b",\"RejectedRoster\":");
        self.rejected_roster.write_json(w);
        w.raw(b",\"FlagSet\":");
        self.flag_set.write_json(w);
        w.raw(b",\"AssistMulti\":");
        self.assist_multi.write_json(w);
        w.raw(b",\"AssistExtraTotal\":");
        self.assist_extra_total.write_json(w);
        w.raw(b",\"AssistFieldDisagree\":");
        self.assist_field_disagree.write_json(w);
        w.raw(b",\"KillerPctOver100\":");
        self.killer_pct_over100.write_json(w);
        w.raw(b",\"AssistPctOver100\":");
        self.assist_pct_over100.write_json(w);
        w.raw(b",\"ParLaFenetre\":");
        self.par_la_fenetre.write_json(w);
        w.raw(b",\"Gate15\":");
        self.gate15.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsBotEntry {
    pub slot: i64,
    pub bot_id: i64,
    pub name: Vec<u8>,
}
impl FactsJsonDecode for FactsKillsBotEntry {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.BotEntry",
        fields: &[
            ("Slot", &Shape::Signed(64, "int")),
            ("BotID", &Shape::Signed(64, "int")),
            ("Name", &Shape::String("string")),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            slot: FactsJsonDecode::from_cell(cell.field("Slot")),
            bot_id: FactsJsonDecode::from_cell(cell.field("BotID")),
            name: raw_string(cell.field("Name")),
        }
    }
}
impl FactsJsonValue for FactsKillsBotEntry {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Slot\":");
        self.slot.write_json(w);
        w.raw(b",\"BotID\":");
        self.bot_id.write_json(w);
        w.raw(b",\"Name\":");
        w.string(&self.name);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsCoverage {
    pub covered: i64,
    pub real_pairs: i64,
    pub reconstructed_pairs: i64,
    pub ghost_pairs: i64,
    pub same_instant_pairs: i64,
    pub feed_kills: i64,
    pub feed_deaths: i64,
    pub bot_deaths: i64,
    pub bot_killer_deaths: i64,
}
impl FactsJsonDecode for FactsKillsCoverage {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.Coverage",
        fields: &[
            ("Covered", &Shape::Signed(64, "int")),
            ("RealPairs", &Shape::Signed(64, "int")),
            ("ReconstructedPairs", &Shape::Signed(64, "int")),
            ("GhostPairs", &Shape::Signed(64, "int")),
            ("SameInstantPairs", &Shape::Signed(64, "int")),
            ("FeedKills", &Shape::Signed(64, "int")),
            ("FeedDeaths", &Shape::Signed(64, "int")),
            ("BotDeaths", &Shape::Signed(64, "int")),
            ("BotKillerDeaths", &Shape::Signed(64, "int")),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            covered: FactsJsonDecode::from_cell(cell.field("Covered")),
            real_pairs: FactsJsonDecode::from_cell(cell.field("RealPairs")),
            reconstructed_pairs: FactsJsonDecode::from_cell(cell.field("ReconstructedPairs")),
            ghost_pairs: FactsJsonDecode::from_cell(cell.field("GhostPairs")),
            same_instant_pairs: FactsJsonDecode::from_cell(cell.field("SameInstantPairs")),
            feed_kills: FactsJsonDecode::from_cell(cell.field("FeedKills")),
            feed_deaths: FactsJsonDecode::from_cell(cell.field("FeedDeaths")),
            bot_deaths: FactsJsonDecode::from_cell(cell.field("BotDeaths")),
            bot_killer_deaths: FactsJsonDecode::from_cell(cell.field("BotKillerDeaths")),
        }
    }
}
impl FactsJsonValue for FactsKillsCoverage {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Covered\":");
        self.covered.write_json(w);
        w.raw(b",\"RealPairs\":");
        self.real_pairs.write_json(w);
        w.raw(b",\"ReconstructedPairs\":");
        self.reconstructed_pairs.write_json(w);
        w.raw(b",\"GhostPairs\":");
        self.ghost_pairs.write_json(w);
        w.raw(b",\"SameInstantPairs\":");
        self.same_instant_pairs.write_json(w);
        w.raw(b",\"FeedKills\":");
        self.feed_kills.write_json(w);
        w.raw(b",\"FeedDeaths\":");
        self.feed_deaths.write_json(w);
        w.raw(b",\"BotDeaths\":");
        self.bot_deaths.write_json(w);
        w.raw(b",\"BotKillerDeaths\":");
        self.bot_killer_deaths.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsDamageShare {
    pub pct: i64,
    pub known: bool,
}
impl FactsJsonDecode for FactsKillsDamageShare {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.DamageShare",
        fields: &[
            ("Pct", &Shape::Signed(64, "int")),
            ("Known", &<bool as FactsJsonDecode>::SHAPE),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            pct: FactsJsonDecode::from_cell(cell.field("Pct")),
            known: FactsJsonDecode::from_cell(cell.field("Known")),
        }
    }
}
impl FactsJsonValue for FactsKillsDamageShare {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Pct\":");
        self.pct.write_json(w);
        w.raw(b",\"Known\":");
        self.known.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsFeedTruth {
    pub killer: Vec<u8>,
    pub present: bool,
}
impl FactsJsonDecode for FactsKillsFeedTruth {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.FeedTruth",
        fields: &[
            ("Killer", &Shape::String("string")),
            ("Present", &<bool as FactsJsonDecode>::SHAPE),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            killer: raw_string(cell.field("Killer")),
            present: FactsJsonDecode::from_cell(cell.field("Present")),
        }
    }
}
impl FactsJsonValue for FactsKillsFeedTruth {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Killer\":");
        w.string(&self.killer);
        w.raw(b",\"Present\":");
        self.present.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsFilmTablePinning {
    pub refusal: Vec<u8>,
    pub build: Vec<u8>,
    pub seats: i64,
    pub pinned: i64,
    pub added_names: i64,
    pub bot_conflict: i64,
    pub duplicate_name: i64,
    pub out_of_range: i64,
    pub inferred: i64,
    pub free_names: i64,
    pub motif_pinned: i64,
    pub motif_agree: i64,
    pub motif_contradict: i64,
    pub motif_duplicate: i64,
    pub motif_readings: i64,
    pub motif_disagreements: i64,
    pub motif_absent: i64,
    pub agree: i64,
    pub contradict: i64,
    pub silent: i64,
}
impl FactsJsonDecode for FactsKillsFilmTablePinning {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.FilmTablePinning",
        fields: &[
            ("Refusal", &Shape::String("killsource.FilmTableRefusal")),
            ("Build", &Shape::String("string")),
            ("Seats", &Shape::Signed(64, "int")),
            ("Pinned", &Shape::Signed(64, "int")),
            ("AddedNames", &Shape::Signed(64, "int")),
            ("BotConflict", &Shape::Signed(64, "int")),
            ("DuplicateName", &Shape::Signed(64, "int")),
            ("OutOfRange", &Shape::Signed(64, "int")),
            ("Inferred", &Shape::Signed(64, "int")),
            ("FreeNames", &Shape::Signed(64, "int")),
            ("MotifPinned", &Shape::Signed(64, "int")),
            ("MotifAgree", &Shape::Signed(64, "int")),
            ("MotifContradict", &Shape::Signed(64, "int")),
            ("MotifDuplicate", &Shape::Signed(64, "int")),
            ("MotifReadings", &Shape::Signed(64, "int")),
            ("MotifDisagreements", &Shape::Signed(64, "int")),
            ("MotifAbsent", &Shape::Signed(64, "int")),
            ("Agree", &Shape::Signed(64, "int")),
            ("Contradict", &Shape::Signed(64, "int")),
            ("Silent", &Shape::Signed(64, "int")),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            refusal: raw_string(cell.field("Refusal")),
            build: raw_string(cell.field("Build")),
            seats: FactsJsonDecode::from_cell(cell.field("Seats")),
            pinned: FactsJsonDecode::from_cell(cell.field("Pinned")),
            added_names: FactsJsonDecode::from_cell(cell.field("AddedNames")),
            bot_conflict: FactsJsonDecode::from_cell(cell.field("BotConflict")),
            duplicate_name: FactsJsonDecode::from_cell(cell.field("DuplicateName")),
            out_of_range: FactsJsonDecode::from_cell(cell.field("OutOfRange")),
            inferred: FactsJsonDecode::from_cell(cell.field("Inferred")),
            free_names: FactsJsonDecode::from_cell(cell.field("FreeNames")),
            motif_pinned: FactsJsonDecode::from_cell(cell.field("MotifPinned")),
            motif_agree: FactsJsonDecode::from_cell(cell.field("MotifAgree")),
            motif_contradict: FactsJsonDecode::from_cell(cell.field("MotifContradict")),
            motif_duplicate: FactsJsonDecode::from_cell(cell.field("MotifDuplicate")),
            motif_readings: FactsJsonDecode::from_cell(cell.field("MotifReadings")),
            motif_disagreements: FactsJsonDecode::from_cell(cell.field("MotifDisagreements")),
            motif_absent: FactsJsonDecode::from_cell(cell.field("MotifAbsent")),
            agree: FactsJsonDecode::from_cell(cell.field("Agree")),
            contradict: FactsJsonDecode::from_cell(cell.field("Contradict")),
            silent: FactsJsonDecode::from_cell(cell.field("Silent")),
        }
    }
}
impl FactsJsonValue for FactsKillsFilmTablePinning {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Refusal\":");
        w.string(&self.refusal);
        w.raw(b",\"Build\":");
        w.string(&self.build);
        w.raw(b",\"Seats\":");
        self.seats.write_json(w);
        w.raw(b",\"Pinned\":");
        self.pinned.write_json(w);
        w.raw(b",\"AddedNames\":");
        self.added_names.write_json(w);
        w.raw(b",\"BotConflict\":");
        self.bot_conflict.write_json(w);
        w.raw(b",\"DuplicateName\":");
        self.duplicate_name.write_json(w);
        w.raw(b",\"OutOfRange\":");
        self.out_of_range.write_json(w);
        w.raw(b",\"Inferred\":");
        self.inferred.write_json(w);
        w.raw(b",\"FreeNames\":");
        self.free_names.write_json(w);
        w.raw(b",\"MotifPinned\":");
        self.motif_pinned.write_json(w);
        w.raw(b",\"MotifAgree\":");
        self.motif_agree.write_json(w);
        w.raw(b",\"MotifContradict\":");
        self.motif_contradict.write_json(w);
        w.raw(b",\"MotifDuplicate\":");
        self.motif_duplicate.write_json(w);
        w.raw(b",\"MotifReadings\":");
        self.motif_readings.write_json(w);
        w.raw(b",\"MotifDisagreements\":");
        self.motif_disagreements.write_json(w);
        w.raw(b",\"MotifAbsent\":");
        self.motif_absent.write_json(w);
        w.raw(b",\"Agree\":");
        self.agree.write_json(w);
        w.raw(b",\"Contradict\":");
        self.contradict.write_json(w);
        w.raw(b",\"Silent\":");
        self.silent.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsKill {
    pub time_ms: i64,
    pub victim: Vec<u8>,
    pub feed: FactsKillsFeedTruth,
    pub source: FactsKillsSourceTruth,
    pub diverges: bool,
    pub read: FactsKillsProvenance,
    pub assist: FactsKillsAssist,
    pub killer_damage: FactsKillsDamageShare,
    pub assist_damage: FactsKillsDamageShare,
}
impl FactsJsonDecode for FactsKillsKill {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.Kill",
        fields: &[
            ("TimeMS", &Shape::Signed(64, "int")),
            ("Victim", &Shape::String("string")),
            ("Feed", &<FactsKillsFeedTruth as FactsJsonDecode>::SHAPE),
            ("Source", &<FactsKillsSourceTruth as FactsJsonDecode>::SHAPE),
            ("Diverges", &<bool as FactsJsonDecode>::SHAPE),
            ("Read", &<FactsKillsProvenance as FactsJsonDecode>::SHAPE),
            ("Assist", &<FactsKillsAssist as FactsJsonDecode>::SHAPE),
            (
                "KillerDamage",
                &<FactsKillsDamageShare as FactsJsonDecode>::SHAPE,
            ),
            (
                "AssistDamage",
                &<FactsKillsDamageShare as FactsJsonDecode>::SHAPE,
            ),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            time_ms: FactsJsonDecode::from_cell(cell.field("TimeMS")),
            victim: raw_string(cell.field("Victim")),
            feed: FactsJsonDecode::from_cell(cell.field("Feed")),
            source: FactsJsonDecode::from_cell(cell.field("Source")),
            diverges: FactsJsonDecode::from_cell(cell.field("Diverges")),
            read: FactsJsonDecode::from_cell(cell.field("Read")),
            assist: FactsJsonDecode::from_cell(cell.field("Assist")),
            killer_damage: FactsJsonDecode::from_cell(cell.field("KillerDamage")),
            assist_damage: FactsJsonDecode::from_cell(cell.field("AssistDamage")),
        }
    }
}
impl FactsJsonValue for FactsKillsKill {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"TimeMS\":");
        self.time_ms.write_json(w);
        w.raw(b",\"Victim\":");
        w.string(&self.victim);
        w.raw(b",\"Feed\":");
        self.feed.write_json(w);
        w.raw(b",\"Source\":");
        self.source.write_json(w);
        w.raw(b",\"Diverges\":");
        self.diverges.write_json(w);
        w.raw(b",\"Read\":");
        self.read.write_json(w);
        w.raw(b",\"Assist\":");
        self.assist.write_json(w);
        w.raw(b",\"KillerDamage\":");
        self.killer_damage.write_json(w);
        w.raw(b",\"AssistDamage\":");
        self.assist_damage.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsPathStats {
    pub population: i64,
    pub matched: i64,
    pub published: i64,
}
impl FactsJsonDecode for FactsKillsPathStats {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.PathStats",
        fields: &[
            ("Population", &Shape::Signed(64, "int")),
            ("Matched", &Shape::Signed(64, "int")),
            ("Published", &Shape::Signed(64, "int")),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            population: FactsJsonDecode::from_cell(cell.field("Population")),
            matched: FactsJsonDecode::from_cell(cell.field("Matched")),
            published: FactsJsonDecode::from_cell(cell.field("Published")),
        }
    }
}
impl FactsJsonValue for FactsKillsPathStats {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Population\":");
        self.population.write_json(w);
        w.raw(b",\"Matched\":");
        self.matched.write_json(w);
        w.raw(b",\"Published\":");
        self.published.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsProvenance {
    pub path: Vec<u8>,
    pub origin: Vec<u8>,
    pub multiplicity: i64,
}
impl FactsJsonDecode for FactsKillsProvenance {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.Provenance",
        fields: &[
            ("Path", &Shape::String("killsource.Path")),
            ("Origin", &Shape::String("killsource.Origin")),
            ("Multiplicity", &Shape::Signed(64, "int")),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            path: raw_string(cell.field("Path")),
            origin: raw_string(cell.field("Origin")),
            multiplicity: FactsJsonDecode::from_cell(cell.field("Multiplicity")),
        }
    }
}
impl FactsJsonValue for FactsKillsProvenance {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Path\":");
        w.string(&self.path);
        w.raw(b",\"Origin\":");
        w.string(&self.origin);
        w.raw(b",\"Multiplicity\":");
        self.multiplicity.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsRelaxedProbe {
    pub candidates: i64,
    pub out_of_catalogue: i64,
    pub paired: i64,
    pub uncovered: i64,
    pub tags: Option<Vec<u32>>,
}
impl FactsJsonDecode for FactsKillsRelaxedProbe {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.RelaxedProbe",
        fields: &[
            ("Candidates", &Shape::Signed(64, "int")),
            ("OutOfCatalogue", &Shape::Signed(64, "int")),
            ("Paired", &Shape::Signed(64, "int")),
            ("Uncovered", &Shape::Signed(64, "int")),
            ("Tags", &Shape::Slice(&<u32 as FactsJsonDecode>::SHAPE)),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            candidates: FactsJsonDecode::from_cell(cell.field("Candidates")),
            out_of_catalogue: FactsJsonDecode::from_cell(cell.field("OutOfCatalogue")),
            paired: FactsJsonDecode::from_cell(cell.field("Paired")),
            uncovered: FactsJsonDecode::from_cell(cell.field("Uncovered")),
            tags: FactsJsonDecode::from_cell(cell.field("Tags")),
        }
    }
}
impl FactsJsonValue for FactsKillsRelaxedProbe {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Candidates\":");
        self.candidates.write_json(w);
        w.raw(b",\"OutOfCatalogue\":");
        self.out_of_catalogue.write_json(w);
        w.raw(b",\"Paired\":");
        self.paired.write_json(w);
        w.raw(b",\"Uncovered\":");
        self.uncovered.write_json(w);
        w.raw(b",\"Tags\":");
        self.tags.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsResult {
    pub kills: Option<Vec<FactsKillsKill>>,
    pub unclaimed_deaths: Option<Vec<FactsKillsUnclaimedDeath>>,
    pub coverage: FactsKillsCoverage,
    pub health: FactsKillsKillSourceHealth,
    pub stats: FactsKillsStats,
    pub roster: FactsKillsRoster,
    pub calibration: Vec<u8>,
    pub profil_calibre: FactsScanProfile,
    pub bijection_margin: i64,
    pub bijection_determined: bool,
    pub probe: Option<FactsKillsRelaxedProbe>,
}
impl FactsJsonDecode for FactsKillsResult {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.Result",
        fields: &[
            (
                "Kills",
                &Shape::Slice(&<FactsKillsKill as FactsJsonDecode>::SHAPE),
            ),
            (
                "UnclaimedDeaths",
                &Shape::Slice(&<FactsKillsUnclaimedDeath as FactsJsonDecode>::SHAPE),
            ),
            ("Coverage", &<FactsKillsCoverage as FactsJsonDecode>::SHAPE),
            (
                "Health",
                &<FactsKillsKillSourceHealth as FactsJsonDecode>::SHAPE,
            ),
            ("Stats", &<FactsKillsStats as FactsJsonDecode>::SHAPE),
            ("Roster", &<FactsKillsRoster as FactsJsonDecode>::SHAPE),
            ("Calibration", &Shape::String("string")),
            (
                "ProfilCalibre",
                &<FactsScanProfile as FactsJsonDecode>::SHAPE,
            ),
            ("BijectionMargin", &Shape::Signed(64, "int")),
            ("BijectionDetermined", &<bool as FactsJsonDecode>::SHAPE),
            (
                "Probe",
                &Shape::Pointer(&<FactsKillsRelaxedProbe as FactsJsonDecode>::SHAPE),
            ),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            kills: FactsJsonDecode::from_cell(cell.field("Kills")),
            unclaimed_deaths: FactsJsonDecode::from_cell(cell.field("UnclaimedDeaths")),
            coverage: FactsJsonDecode::from_cell(cell.field("Coverage")),
            health: FactsJsonDecode::from_cell(cell.field("Health")),
            stats: FactsJsonDecode::from_cell(cell.field("Stats")),
            roster: FactsJsonDecode::from_cell(cell.field("Roster")),
            calibration: raw_string(cell.field("Calibration")),
            profil_calibre: FactsJsonDecode::from_cell(cell.field("ProfilCalibre")),
            bijection_margin: FactsJsonDecode::from_cell(cell.field("BijectionMargin")),
            bijection_determined: FactsJsonDecode::from_cell(cell.field("BijectionDetermined")),
            probe: <JsonPointer<FactsKillsRelaxedProbe> as FactsJsonDecode>::from_cell(
                cell.field("Probe"),
            )
            .0,
        }
    }
}
impl FactsJsonValue for FactsKillsResult {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Kills\":");
        self.kills.write_json(w);
        w.raw(b",\"UnclaimedDeaths\":");
        self.unclaimed_deaths.write_json(w);
        w.raw(b",\"Coverage\":");
        self.coverage.write_json(w);
        w.raw(b",\"Health\":");
        self.health.write_json(w);
        w.raw(b",\"Stats\":");
        self.stats.write_json(w);
        w.raw(b",\"Roster\":");
        self.roster.write_json(w);
        w.raw(b",\"Calibration\":");
        w.string(&self.calibration);
        w.raw(b",\"ProfilCalibre\":");
        self.profil_calibre.write_json(w);
        w.raw(b",\"BijectionMargin\":");
        self.bijection_margin.write_json(w);
        w.raw(b",\"BijectionDetermined\":");
        self.bijection_determined.write_json(w);
        w.raw(b",\"Probe\":");
        self.probe.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsRoster {
    pub names: Option<Vec<Vec<u8>>>,
    pub humans: i64,
    pub bots: Option<Vec<FactsKillsBotEntry>>,
    pub index_to_name: Option<Vec<Vec<u8>>>,
    pub unpinned_bots: Option<Vec<FactsKillsBotEntry>>,
    pub bots_succedes: i64,
    pub index_source: Option<Vec<Vec<u8>>>,
    pub film_table: FactsKillsFilmTablePinning,
}
impl FactsJsonDecode for FactsKillsRoster {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.Roster",
        fields: &[
            ("Names", &Shape::Slice(&Shape::String("string"))),
            ("Humans", &Shape::Signed(64, "int")),
            (
                "Bots",
                &Shape::Slice(&<FactsKillsBotEntry as FactsJsonDecode>::SHAPE),
            ),
            ("IndexToName", &Shape::Slice(&Shape::String("string"))),
            (
                "UnpinnedBots",
                &Shape::Slice(&<FactsKillsBotEntry as FactsJsonDecode>::SHAPE),
            ),
            ("BotsSuccedes", &Shape::Signed(64, "int")),
            (
                "IndexSource",
                &Shape::Slice(&Shape::String("killsource.IndexOrigin")),
            ),
            (
                "FilmTable",
                &<FactsKillsFilmTablePinning as FactsJsonDecode>::SHAPE,
            ),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            names: raw_strings(cell.field("Names")),
            humans: FactsJsonDecode::from_cell(cell.field("Humans")),
            bots: FactsJsonDecode::from_cell(cell.field("Bots")),
            index_to_name: raw_strings(cell.field("IndexToName")),
            unpinned_bots: FactsJsonDecode::from_cell(cell.field("UnpinnedBots")),
            bots_succedes: FactsJsonDecode::from_cell(cell.field("BotsSuccedes")),
            index_source: raw_strings(cell.field("IndexSource")),
            film_table: FactsJsonDecode::from_cell(cell.field("FilmTable")),
        }
    }
}
impl FactsJsonValue for FactsKillsRoster {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Names\":");
        write_strings(&self.names, w);
        w.raw(b",\"Humans\":");
        self.humans.write_json(w);
        w.raw(b",\"Bots\":");
        self.bots.write_json(w);
        w.raw(b",\"IndexToName\":");
        write_strings(&self.index_to_name, w);
        w.raw(b",\"UnpinnedBots\":");
        self.unpinned_bots.write_json(w);
        w.raw(b",\"BotsSuccedes\":");
        self.bots_succedes.write_json(w);
        w.raw(b",\"IndexSource\":");
        write_strings(&self.index_source, w);
        w.raw(b",\"FilmTable\":");
        self.film_table.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsSourceTruth {
    pub tag: u32,
    pub display: Vec<u8>,
    pub named: bool,
    pub class: Vec<u8>,
    pub status: Vec<u8>,
    pub detail: Vec<u8>,
    pub reserve: Vec<u8>,
    pub category: i64,
}
impl FactsJsonDecode for FactsKillsSourceTruth {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.SourceTruth",
        fields: &[
            ("Tag", &<u32 as FactsJsonDecode>::SHAPE),
            ("Display", &Shape::String("string")),
            ("Named", &<bool as FactsJsonDecode>::SHAPE),
            ("Class", &Shape::String("damagetag.Class")),
            ("Status", &Shape::String("damagetag.Status")),
            ("Detail", &Shape::String("string")),
            ("Reserve", &Shape::String("string")),
            ("Category", &Shape::Signed(64, "killsource.Category")),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            tag: FactsJsonDecode::from_cell(cell.field("Tag")),
            display: raw_string(cell.field("Display")),
            named: FactsJsonDecode::from_cell(cell.field("Named")),
            class: raw_string(cell.field("Class")),
            status: raw_string(cell.field("Status")),
            detail: raw_string(cell.field("Detail")),
            reserve: raw_string(cell.field("Reserve")),
            category: FactsJsonDecode::from_cell(cell.field("Category")),
        }
    }
}
impl FactsJsonValue for FactsKillsSourceTruth {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Tag\":");
        self.tag.write_json(w);
        w.raw(b",\"Display\":");
        w.string(&self.display);
        w.raw(b",\"Named\":");
        self.named.write_json(w);
        w.raw(b",\"Class\":");
        w.string(&self.class);
        w.raw(b",\"Status\":");
        w.string(&self.status);
        w.raw(b",\"Detail\":");
        w.string(&self.detail);
        w.raw(b",\"Reserve\":");
        w.string(&self.reserve);
        w.raw(b",\"Category\":");
        self.category.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsStats {
    pub walk: FactsKillsPathStats,
    pub scan: FactsKillsPathStats,
    pub self_walk: FactsKillsPathStats,
    pub self_scan: FactsKillsPathStats,
    pub bot: FactsKillsPathStats,
    pub bot_killer: FactsKillsPathStats,
    pub unclaimed: FactsKillsPathStats,
    pub redundant: i64,
    pub no_bit: i64,
    pub agree: i64,
    pub disagree: i64,
    pub multi_candidate: i64,
    pub packets_with_events: i64,
    pub packets_located: i64,
    pub assist: FactsKillsAssistStats,
    pub couples: FactsKillsCoupleStats,
    pub appariement: FactsKillsApparStats,
}
impl FactsJsonDecode for FactsKillsStats {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.Stats",
        fields: &[
            ("Walk", &<FactsKillsPathStats as FactsJsonDecode>::SHAPE),
            ("Scan", &<FactsKillsPathStats as FactsJsonDecode>::SHAPE),
            ("SelfWalk", &<FactsKillsPathStats as FactsJsonDecode>::SHAPE),
            ("SelfScan", &<FactsKillsPathStats as FactsJsonDecode>::SHAPE),
            ("Bot", &<FactsKillsPathStats as FactsJsonDecode>::SHAPE),
            (
                "BotKiller",
                &<FactsKillsPathStats as FactsJsonDecode>::SHAPE,
            ),
            (
                "Unclaimed",
                &<FactsKillsPathStats as FactsJsonDecode>::SHAPE,
            ),
            ("Redundant", &Shape::Signed(64, "int")),
            ("NoBit", &Shape::Signed(64, "int")),
            ("Agree", &Shape::Signed(64, "int")),
            ("Disagree", &Shape::Signed(64, "int")),
            ("MultiCandidate", &Shape::Signed(64, "int")),
            ("PacketsWithEvents", &Shape::Signed(64, "int")),
            ("PacketsLocated", &Shape::Signed(64, "int")),
            ("Assist", &<FactsKillsAssistStats as FactsJsonDecode>::SHAPE),
            (
                "Couples",
                &<FactsKillsCoupleStats as FactsJsonDecode>::SHAPE,
            ),
            (
                "Appariement",
                &<FactsKillsApparStats as FactsJsonDecode>::SHAPE,
            ),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            walk: FactsJsonDecode::from_cell(cell.field("Walk")),
            scan: FactsJsonDecode::from_cell(cell.field("Scan")),
            self_walk: FactsJsonDecode::from_cell(cell.field("SelfWalk")),
            self_scan: FactsJsonDecode::from_cell(cell.field("SelfScan")),
            bot: FactsJsonDecode::from_cell(cell.field("Bot")),
            bot_killer: FactsJsonDecode::from_cell(cell.field("BotKiller")),
            unclaimed: FactsJsonDecode::from_cell(cell.field("Unclaimed")),
            redundant: FactsJsonDecode::from_cell(cell.field("Redundant")),
            no_bit: FactsJsonDecode::from_cell(cell.field("NoBit")),
            agree: FactsJsonDecode::from_cell(cell.field("Agree")),
            disagree: FactsJsonDecode::from_cell(cell.field("Disagree")),
            multi_candidate: FactsJsonDecode::from_cell(cell.field("MultiCandidate")),
            packets_with_events: FactsJsonDecode::from_cell(cell.field("PacketsWithEvents")),
            packets_located: FactsJsonDecode::from_cell(cell.field("PacketsLocated")),
            assist: FactsJsonDecode::from_cell(cell.field("Assist")),
            couples: FactsJsonDecode::from_cell(cell.field("Couples")),
            appariement: FactsJsonDecode::from_cell(cell.field("Appariement")),
        }
    }
}
impl FactsJsonValue for FactsKillsStats {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Walk\":");
        self.walk.write_json(w);
        w.raw(b",\"Scan\":");
        self.scan.write_json(w);
        w.raw(b",\"SelfWalk\":");
        self.self_walk.write_json(w);
        w.raw(b",\"SelfScan\":");
        self.self_scan.write_json(w);
        w.raw(b",\"Bot\":");
        self.bot.write_json(w);
        w.raw(b",\"BotKiller\":");
        self.bot_killer.write_json(w);
        w.raw(b",\"Unclaimed\":");
        self.unclaimed.write_json(w);
        w.raw(b",\"Redundant\":");
        self.redundant.write_json(w);
        w.raw(b",\"NoBit\":");
        self.no_bit.write_json(w);
        w.raw(b",\"Agree\":");
        self.agree.write_json(w);
        w.raw(b",\"Disagree\":");
        self.disagree.write_json(w);
        w.raw(b",\"MultiCandidate\":");
        self.multi_candidate.write_json(w);
        w.raw(b",\"PacketsWithEvents\":");
        self.packets_with_events.write_json(w);
        w.raw(b",\"PacketsLocated\":");
        self.packets_located.write_json(w);
        w.raw(b",\"Assist\":");
        self.assist.write_json(w);
        w.raw(b",\"Couples\":");
        self.couples.write_json(w);
        w.raw(b",\"Appariement\":");
        self.appariement.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsUnclaimedDeath {
    pub time_ms: i64,
    pub victim: Vec<u8>,
    pub victim_xuid: u64,
    pub source: FactsKillsSourceTruth,
    pub read: FactsKillsProvenance,
}
impl FactsJsonDecode for FactsKillsUnclaimedDeath {
    const SHAPE: Shape = Shape::Struct {
        name: "killsource.UnclaimedDeath",
        fields: &[
            ("TimeMS", &Shape::Signed(64, "int")),
            ("Victim", &Shape::String("string")),
            ("VictimXUID", &Shape::Unsigned(64, "uint64")),
            ("Source", &<FactsKillsSourceTruth as FactsJsonDecode>::SHAPE),
            ("Read", &<FactsKillsProvenance as FactsJsonDecode>::SHAPE),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            time_ms: FactsJsonDecode::from_cell(cell.field("TimeMS")),
            victim: raw_string(cell.field("Victim")),
            victim_xuid: FactsJsonDecode::from_cell(cell.field("VictimXUID")),
            source: FactsJsonDecode::from_cell(cell.field("Source")),
            read: FactsJsonDecode::from_cell(cell.field("Read")),
        }
    }
}
impl FactsJsonValue for FactsKillsUnclaimedDeath {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"TimeMS\":");
        self.time_ms.write_json(w);
        w.raw(b",\"Victim\":");
        w.string(&self.victim);
        w.raw(b",\"VictimXUID\":");
        self.victim_xuid.write_json(w);
        w.raw(b",\"Source\":");
        self.source.write_json(w);
        w.raw(b",\"Read\":");
        self.read.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsApparStats {
    pub identite: i64,
    pub fenetre: i64,
    pub bot_fenetre: i64,
    pub non_revendiquee_fenetre: i64,
    pub couples_sans_identite: i64,
}
impl FactsJsonDecode for FactsKillsApparStats {
    const SHAPE: Shape = Shape::Struct {
        name: "types.ApparStats",
        fields: &[
            ("Identite", &Shape::Signed(64, "int")),
            ("Fenetre", &Shape::Signed(64, "int")),
            ("BotFenetre", &Shape::Signed(64, "int")),
            ("NonRevendiqueeFenetre", &Shape::Signed(64, "int")),
            ("CouplesSansIdentite", &Shape::Signed(64, "int")),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            identite: FactsJsonDecode::from_cell(cell.field("Identite")),
            fenetre: FactsJsonDecode::from_cell(cell.field("Fenetre")),
            bot_fenetre: FactsJsonDecode::from_cell(cell.field("BotFenetre")),
            non_revendiquee_fenetre: FactsJsonDecode::from_cell(
                cell.field("NonRevendiqueeFenetre"),
            ),
            couples_sans_identite: FactsJsonDecode::from_cell(cell.field("CouplesSansIdentite")),
        }
    }
}
impl FactsJsonValue for FactsKillsApparStats {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Identite\":");
        self.identite.write_json(w);
        w.raw(b",\"Fenetre\":");
        self.fenetre.write_json(w);
        w.raw(b",\"BotFenetre\":");
        self.bot_fenetre.write_json(w);
        w.raw(b",\"NonRevendiqueeFenetre\":");
        self.non_revendiquee_fenetre.write_json(w);
        w.raw(b",\"CouplesSansIdentite\":");
        self.couples_sans_identite.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsAssist {
    pub name: Vec<u8>,
    pub index: i64,
    pub rejected: Vec<u8>,
    pub known: bool,
    pub extra: i64,
}
impl FactsJsonDecode for FactsKillsAssist {
    const SHAPE: Shape = Shape::Struct {
        name: "types.Assist",
        fields: &[
            ("Name", &Shape::String("string")),
            ("Index", &Shape::Signed(64, "int")),
            ("Rejected", &Shape::String("string")),
            ("Known", &<bool as FactsJsonDecode>::SHAPE),
            ("Extra", &Shape::Signed(64, "int")),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            name: raw_string(cell.field("Name")),
            index: FactsJsonDecode::from_cell(cell.field("Index")),
            rejected: raw_string(cell.field("Rejected")),
            known: FactsJsonDecode::from_cell(cell.field("Known")),
            extra: FactsJsonDecode::from_cell(cell.field("Extra")),
        }
    }
}
impl FactsJsonValue for FactsKillsAssist {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Name\":");
        w.string(&self.name);
        w.raw(b",\"Index\":");
        self.index.write_json(w);
        w.raw(b",\"Rejected\":");
        w.string(&self.rejected);
        w.raw(b",\"Known\":");
        self.known.write_json(w);
        w.raw(b",\"Extra\":");
        self.extra.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsKillsCoupleStats {
    pub meme_instant: i64,
    pub lus: i64,
    pub recolles: i64,
    pub perdus: i64,
    pub victimes_bot_lues: i64,
    pub muet: i64,
    pub ambigu: i64,
    pub accord: i64,
    pub contradiction: i64,
}
impl FactsJsonDecode for FactsKillsCoupleStats {
    const SHAPE: Shape = Shape::Struct {
        name: "types.CoupleStats",
        fields: &[
            ("MemeInstant", &Shape::Signed(64, "int")),
            ("Lus", &Shape::Signed(64, "int")),
            ("Recolles", &Shape::Signed(64, "int")),
            ("Perdus", &Shape::Signed(64, "int")),
            ("VictimesBotLues", &Shape::Signed(64, "int")),
            ("Muet", &Shape::Signed(64, "int")),
            ("Ambigu", &Shape::Signed(64, "int")),
            ("Accord", &Shape::Signed(64, "int")),
            ("Contradiction", &Shape::Signed(64, "int")),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            meme_instant: FactsJsonDecode::from_cell(cell.field("MemeInstant")),
            lus: FactsJsonDecode::from_cell(cell.field("Lus")),
            recolles: FactsJsonDecode::from_cell(cell.field("Recolles")),
            perdus: FactsJsonDecode::from_cell(cell.field("Perdus")),
            victimes_bot_lues: FactsJsonDecode::from_cell(cell.field("VictimesBotLues")),
            muet: FactsJsonDecode::from_cell(cell.field("Muet")),
            ambigu: FactsJsonDecode::from_cell(cell.field("Ambigu")),
            accord: FactsJsonDecode::from_cell(cell.field("Accord")),
            contradiction: FactsJsonDecode::from_cell(cell.field("Contradiction")),
        }
    }
}
impl FactsJsonValue for FactsKillsCoupleStats {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"MemeInstant\":");
        self.meme_instant.write_json(w);
        w.raw(b",\"Lus\":");
        self.lus.write_json(w);
        w.raw(b",\"Recolles\":");
        self.recolles.write_json(w);
        w.raw(b",\"Perdus\":");
        self.perdus.write_json(w);
        w.raw(b",\"VictimesBotLues\":");
        self.victimes_bot_lues.write_json(w);
        w.raw(b",\"Muet\":");
        self.muet.write_json(w);
        w.raw(b",\"Ambigu\":");
        self.ambigu.write_json(w);
        w.raw(b",\"Accord\":");
        self.accord.write_json(w);
        w.raw(b",\"Contradiction\":");
        self.contradiction.write_json(w);
        w.raw(b"}");
    }
}
pub fn encode_facts_kills_json(value: Option<&FactsKillsResult>) -> Result<Vec<u8>, String> {
    let mut w = FactsJsonWriter::default();
    if let Some(v) = value {
        v.write_json(&mut w);
    } else {
        w.raw(b"null");
    }
    if let Some(e) = w.error {
        Err(e)
    } else {
        Ok(w.bytes)
    }
}
pub fn decode_facts_kills_json(bytes: &[u8]) -> Result<Option<FactsKillsResult>, String> {
    FactsKillsJsonReader::default().read(bytes)
}
/// Retains successful native updates, including pointer and slice backing state.
/// Errors poison the reader; discard it after any failed section.
#[derive(Default)]
pub struct FactsKillsJsonReader {
    state: FactsJsonState<JsonPointer<FactsKillsResult>>,
}
impl FactsKillsJsonReader {
    pub fn read(&mut self, bytes: &[u8]) -> Result<Option<FactsKillsResult>, String> {
        self.state.apply(bytes)?;
        Ok(self.state.value()?.0)
    }
}
