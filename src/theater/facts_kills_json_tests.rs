use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn source(v: &Value) -> Option<FactsKillsResult> {
    if v.is_null() {
        None
    } else {
        Some(FactsKillsResult {
            kills: v["Kills"].as_array().map(|a| {
                a.iter()
                    .map(|v| FactsKillsKill {
                        time_ms: v["TimeMS"].as_i64().unwrap(),
                        victim: v["Victim"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_u64().unwrap() as u8)
                            .collect(),
                        feed: FactsKillsFeedTruth {
                            killer: v["Feed"]["Killer"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            present: v["Feed"]["Present"].as_bool().unwrap(),
                        },
                        source: FactsKillsSourceTruth {
                            tag: v["Source"]["Tag"].as_u64().unwrap() as u32,
                            display: v["Source"]["Display"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            named: v["Source"]["Named"].as_bool().unwrap(),
                            class: v["Source"]["Class"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            status: v["Source"]["Status"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            detail: v["Source"]["Detail"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            reserve: v["Source"]["Reserve"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            category: v["Source"]["Category"].as_i64().unwrap(),
                        },
                        diverges: v["Diverges"].as_bool().unwrap(),
                        read: FactsKillsProvenance {
                            path: v["Read"]["Path"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            origin: v["Read"]["Origin"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            multiplicity: v["Read"]["Multiplicity"].as_i64().unwrap(),
                        },
                        assist: FactsKillsAssist {
                            name: v["Assist"]["Name"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            index: v["Assist"]["Index"].as_i64().unwrap(),
                            rejected: v["Assist"]["Rejected"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            known: v["Assist"]["Known"].as_bool().unwrap(),
                            extra: v["Assist"]["Extra"].as_i64().unwrap(),
                        },
                        killer_damage: FactsKillsDamageShare {
                            pct: v["KillerDamage"]["Pct"].as_i64().unwrap(),
                            known: v["KillerDamage"]["Known"].as_bool().unwrap(),
                        },
                        assist_damage: FactsKillsDamageShare {
                            pct: v["AssistDamage"]["Pct"].as_i64().unwrap(),
                            known: v["AssistDamage"]["Known"].as_bool().unwrap(),
                        },
                    })
                    .collect()
            }),
            unclaimed_deaths: v["UnclaimedDeaths"].as_array().map(|a| {
                a.iter()
                    .map(|v| FactsKillsUnclaimedDeath {
                        time_ms: v["TimeMS"].as_i64().unwrap(),
                        victim: v["Victim"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_u64().unwrap() as u8)
                            .collect(),
                        victim_xuid: v["VictimXUID"].as_u64().unwrap(),
                        source: FactsKillsSourceTruth {
                            tag: v["Source"]["Tag"].as_u64().unwrap() as u32,
                            display: v["Source"]["Display"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            named: v["Source"]["Named"].as_bool().unwrap(),
                            class: v["Source"]["Class"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            status: v["Source"]["Status"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            detail: v["Source"]["Detail"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            reserve: v["Source"]["Reserve"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            category: v["Source"]["Category"].as_i64().unwrap(),
                        },
                        read: FactsKillsProvenance {
                            path: v["Read"]["Path"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            origin: v["Read"]["Origin"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                            multiplicity: v["Read"]["Multiplicity"].as_i64().unwrap(),
                        },
                    })
                    .collect()
            }),
            coverage: FactsKillsCoverage {
                covered: v["Coverage"]["Covered"].as_i64().unwrap(),
                real_pairs: v["Coverage"]["RealPairs"].as_i64().unwrap(),
                reconstructed_pairs: v["Coverage"]["ReconstructedPairs"].as_i64().unwrap(),
                ghost_pairs: v["Coverage"]["GhostPairs"].as_i64().unwrap(),
                same_instant_pairs: v["Coverage"]["SameInstantPairs"].as_i64().unwrap(),
                feed_kills: v["Coverage"]["FeedKills"].as_i64().unwrap(),
                feed_deaths: v["Coverage"]["FeedDeaths"].as_i64().unwrap(),
                bot_deaths: v["Coverage"]["BotDeaths"].as_i64().unwrap(),
                bot_killer_deaths: v["Coverage"]["BotKillerDeaths"].as_i64().unwrap(),
            },
            health: FactsKillsKillSourceHealth {
                film: v["Health"]["Film"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as u8)
                    .collect(),
                candidates: v["Health"]["Candidates"].as_i64().unwrap(),
                published: v["Health"]["Published"].as_i64().unwrap(),
                unexplained_pair: v["Health"]["UnexplainedPair"].as_i64().unwrap(),
                unexplained_self: v["Health"]["UnexplainedSelf"].as_i64().unwrap(),
                unexplained_bot_idx: v["Health"]["UnexplainedBotIdx"].as_i64().unwrap(),
                out_of_roster: v["Health"]["OutOfRoster"].as_i64().unwrap(),
                tag_out_of_catalogue_walk: v["Health"]["TagOutOfCatalogueWalk"].as_i64().unwrap(),
                tag_out_of_catalogue_scan: v["Health"]["TagOutOfCatalogueScan"].as_i64().unwrap(),
                deaths_real: v["Health"]["DeathsReal"].as_i64().unwrap(),
                deaths_covered: v["Health"]["DeathsCovered"].as_i64().unwrap(),
            },
            stats: FactsKillsStats {
                walk: FactsKillsPathStats {
                    population: v["Stats"]["Walk"]["Population"].as_i64().unwrap(),
                    matched: v["Stats"]["Walk"]["Matched"].as_i64().unwrap(),
                    published: v["Stats"]["Walk"]["Published"].as_i64().unwrap(),
                },
                scan: FactsKillsPathStats {
                    population: v["Stats"]["Scan"]["Population"].as_i64().unwrap(),
                    matched: v["Stats"]["Scan"]["Matched"].as_i64().unwrap(),
                    published: v["Stats"]["Scan"]["Published"].as_i64().unwrap(),
                },
                self_walk: FactsKillsPathStats {
                    population: v["Stats"]["SelfWalk"]["Population"].as_i64().unwrap(),
                    matched: v["Stats"]["SelfWalk"]["Matched"].as_i64().unwrap(),
                    published: v["Stats"]["SelfWalk"]["Published"].as_i64().unwrap(),
                },
                self_scan: FactsKillsPathStats {
                    population: v["Stats"]["SelfScan"]["Population"].as_i64().unwrap(),
                    matched: v["Stats"]["SelfScan"]["Matched"].as_i64().unwrap(),
                    published: v["Stats"]["SelfScan"]["Published"].as_i64().unwrap(),
                },
                bot: FactsKillsPathStats {
                    population: v["Stats"]["Bot"]["Population"].as_i64().unwrap(),
                    matched: v["Stats"]["Bot"]["Matched"].as_i64().unwrap(),
                    published: v["Stats"]["Bot"]["Published"].as_i64().unwrap(),
                },
                bot_killer: FactsKillsPathStats {
                    population: v["Stats"]["BotKiller"]["Population"].as_i64().unwrap(),
                    matched: v["Stats"]["BotKiller"]["Matched"].as_i64().unwrap(),
                    published: v["Stats"]["BotKiller"]["Published"].as_i64().unwrap(),
                },
                unclaimed: FactsKillsPathStats {
                    population: v["Stats"]["Unclaimed"]["Population"].as_i64().unwrap(),
                    matched: v["Stats"]["Unclaimed"]["Matched"].as_i64().unwrap(),
                    published: v["Stats"]["Unclaimed"]["Published"].as_i64().unwrap(),
                },
                redundant: v["Stats"]["Redundant"].as_i64().unwrap(),
                no_bit: v["Stats"]["NoBit"].as_i64().unwrap(),
                agree: v["Stats"]["Agree"].as_i64().unwrap(),
                disagree: v["Stats"]["Disagree"].as_i64().unwrap(),
                multi_candidate: v["Stats"]["MultiCandidate"].as_i64().unwrap(),
                packets_with_events: v["Stats"]["PacketsWithEvents"].as_i64().unwrap(),
                packets_located: v["Stats"]["PacketsLocated"].as_i64().unwrap(),
                assist: FactsKillsAssistStats {
                    kill_events: v["Stats"]["Assist"]["KillEvents"].as_i64().unwrap(),
                    attached: v["Stats"]["Assist"]["Attached"].as_i64().unwrap(),
                    multi: v["Stats"]["Assist"]["Multi"].as_i64().unwrap(),
                    named: v["Stats"]["Assist"]["Named"].as_i64().unwrap(),
                    no_assist: v["Stats"]["Assist"]["NoAssist"].as_i64().unwrap(),
                    rejected_self: v["Stats"]["Assist"]["RejectedSelf"].as_i64().unwrap(),
                    rejected_victim: v["Stats"]["Assist"]["RejectedVictim"].as_i64().unwrap(),
                    rejected_roster: v["Stats"]["Assist"]["RejectedRoster"].as_i64().unwrap(),
                    flag_set: v["Stats"]["Assist"]["FlagSet"].as_i64().unwrap(),
                    assist_multi: v["Stats"]["Assist"]["AssistMulti"].as_i64().unwrap(),
                    assist_extra_total: v["Stats"]["Assist"]["AssistExtraTotal"].as_i64().unwrap(),
                    assist_field_disagree: v["Stats"]["Assist"]["AssistFieldDisagree"]
                        .as_i64()
                        .unwrap(),
                    killer_pct_over100: v["Stats"]["Assist"]["KillerPctOver100"].as_i64().unwrap(),
                    assist_pct_over100: v["Stats"]["Assist"]["AssistPctOver100"].as_i64().unwrap(),
                    par_la_fenetre: v["Stats"]["Assist"]["ParLaFenetre"].as_i64().unwrap(),
                    gate15: v["Stats"]["Assist"]["Gate15"].as_bool().unwrap(),
                },
                couples: FactsKillsCoupleStats {
                    meme_instant: v["Stats"]["Couples"]["MemeInstant"].as_i64().unwrap(),
                    lus: v["Stats"]["Couples"]["Lus"].as_i64().unwrap(),
                    recolles: v["Stats"]["Couples"]["Recolles"].as_i64().unwrap(),
                    perdus: v["Stats"]["Couples"]["Perdus"].as_i64().unwrap(),
                    victimes_bot_lues: v["Stats"]["Couples"]["VictimesBotLues"].as_i64().unwrap(),
                    muet: v["Stats"]["Couples"]["Muet"].as_i64().unwrap(),
                    ambigu: v["Stats"]["Couples"]["Ambigu"].as_i64().unwrap(),
                    accord: v["Stats"]["Couples"]["Accord"].as_i64().unwrap(),
                    contradiction: v["Stats"]["Couples"]["Contradiction"].as_i64().unwrap(),
                },
                appariement: FactsKillsApparStats {
                    identite: v["Stats"]["Appariement"]["Identite"].as_i64().unwrap(),
                    fenetre: v["Stats"]["Appariement"]["Fenetre"].as_i64().unwrap(),
                    bot_fenetre: v["Stats"]["Appariement"]["BotFenetre"].as_i64().unwrap(),
                    non_revendiquee_fenetre: v["Stats"]["Appariement"]["NonRevendiqueeFenetre"]
                        .as_i64()
                        .unwrap(),
                    couples_sans_identite: v["Stats"]["Appariement"]["CouplesSansIdentite"]
                        .as_i64()
                        .unwrap(),
                },
            },
            roster: FactsKillsRoster {
                names: v["Roster"]["Names"].as_array().map(|a| {
                    a.iter()
                        .map(|v| {
                            v.as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect()
                        })
                        .collect()
                }),
                humans: v["Roster"]["Humans"].as_i64().unwrap(),
                bots: v["Roster"]["Bots"].as_array().map(|a| {
                    a.iter()
                        .map(|v| FactsKillsBotEntry {
                            slot: v["Slot"].as_i64().unwrap(),
                            bot_id: v["BotID"].as_i64().unwrap(),
                            name: v["Name"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                        })
                        .collect()
                }),
                index_to_name: v["Roster"]["IndexToName"].as_array().map(|a| {
                    a.iter()
                        .map(|v| {
                            v.as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect()
                        })
                        .collect()
                }),
                unpinned_bots: v["Roster"]["UnpinnedBots"].as_array().map(|a| {
                    a.iter()
                        .map(|v| FactsKillsBotEntry {
                            slot: v["Slot"].as_i64().unwrap(),
                            bot_id: v["BotID"].as_i64().unwrap(),
                            name: v["Name"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect(),
                        })
                        .collect()
                }),
                bots_succedes: v["Roster"]["BotsSuccedes"].as_i64().unwrap(),
                index_source: v["Roster"]["IndexSource"].as_array().map(|a| {
                    a.iter()
                        .map(|v| {
                            v.as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().unwrap() as u8)
                                .collect()
                        })
                        .collect()
                }),
                film_table: FactsKillsFilmTablePinning {
                    refusal: v["Roster"]["FilmTable"]["Refusal"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_u64().unwrap() as u8)
                        .collect(),
                    build: v["Roster"]["FilmTable"]["Build"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_u64().unwrap() as u8)
                        .collect(),
                    seats: v["Roster"]["FilmTable"]["Seats"].as_i64().unwrap(),
                    pinned: v["Roster"]["FilmTable"]["Pinned"].as_i64().unwrap(),
                    added_names: v["Roster"]["FilmTable"]["AddedNames"].as_i64().unwrap(),
                    bot_conflict: v["Roster"]["FilmTable"]["BotConflict"].as_i64().unwrap(),
                    duplicate_name: v["Roster"]["FilmTable"]["DuplicateName"].as_i64().unwrap(),
                    out_of_range: v["Roster"]["FilmTable"]["OutOfRange"].as_i64().unwrap(),
                    inferred: v["Roster"]["FilmTable"]["Inferred"].as_i64().unwrap(),
                    free_names: v["Roster"]["FilmTable"]["FreeNames"].as_i64().unwrap(),
                    motif_pinned: v["Roster"]["FilmTable"]["MotifPinned"].as_i64().unwrap(),
                    motif_agree: v["Roster"]["FilmTable"]["MotifAgree"].as_i64().unwrap(),
                    motif_contradict: v["Roster"]["FilmTable"]["MotifContradict"]
                        .as_i64()
                        .unwrap(),
                    motif_duplicate: v["Roster"]["FilmTable"]["MotifDuplicate"].as_i64().unwrap(),
                    motif_readings: v["Roster"]["FilmTable"]["MotifReadings"].as_i64().unwrap(),
                    motif_disagreements: v["Roster"]["FilmTable"]["MotifDisagreements"]
                        .as_i64()
                        .unwrap(),
                    motif_absent: v["Roster"]["FilmTable"]["MotifAbsent"].as_i64().unwrap(),
                    agree: v["Roster"]["FilmTable"]["Agree"].as_i64().unwrap(),
                    contradict: v["Roster"]["FilmTable"]["Contradict"].as_i64().unwrap(),
                    silent: v["Roster"]["FilmTable"]["Silent"].as_i64().unwrap(),
                },
            },
            calibration: v["Calibration"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as u8)
                .collect(),
            profil_calibre: FactsScanProfile {
                movement: FactsMovementProfile {
                    traversal: FactsPrecisionDescriptor {
                        index_width: v["ProfilCalibre"]["Mouvement"]["Traversal"]["IndexW"]
                            .as_u64()
                            .unwrap(),
                        axis_widths: std::array::from_fn(|i| {
                            v["ProfilCalibre"]["Mouvement"]["Traversal"]["AxisW"][i]
                                .as_u64()
                                .unwrap()
                        }),
                        region: v["ProfilCalibre"]["Mouvement"]["Traversal"]["Region"]
                            .as_u64()
                            .unwrap() as u32,
                    },
                    world_object: FactsPrecisionDescriptor {
                        index_width: v["ProfilCalibre"]["Mouvement"]["WorldObject"]["IndexW"]
                            .as_u64()
                            .unwrap(),
                        axis_widths: std::array::from_fn(|i| {
                            v["ProfilCalibre"]["Mouvement"]["WorldObject"]["AxisW"][i]
                                .as_u64()
                                .unwrap()
                        }),
                        region: v["ProfilCalibre"]["Mouvement"]["WorldObject"]["Region"]
                            .as_u64()
                            .unwrap() as u32,
                    },
                    delta_quantum: f32::from_bits(
                        v["ProfilCalibre"]["Mouvement"]["DeltaQuantum"]
                            .as_u64()
                            .unwrap() as u32,
                    ),
                    delta_axis_width: v["ProfilCalibre"]["Mouvement"]["DeltaAxisWidth"]
                        .as_u64()
                        .unwrap(),
                    range: std::array::from_fn(|i| FactsAxisRange {
                        min: f32::from_bits(
                            v["ProfilCalibre"]["Mouvement"]["Range"][i]["Min"]
                                .as_u64()
                                .unwrap() as u32,
                        ),
                        max: f32::from_bits(
                            v["ProfilCalibre"]["Mouvement"]["Range"][i]["Max"]
                                .as_u64()
                                .unwrap() as u32,
                        ),
                    }),
                    full_precision: v["ProfilCalibre"]["Mouvement"]["FullPrecision"]
                        .as_bool()
                        .unwrap(),
                    delta_has_handle_tail: v["ProfilCalibre"]["Mouvement"]["DeltaHasHandleTail"]
                        .as_bool()
                        .unwrap(),
                    calibrated_skip: v["ProfilCalibre"]["Mouvement"]["CalibratedSkip"]
                        .as_bool()
                        .unwrap(),
                    mobility_action_extra_bits:
                        v["ProfilCalibre"]["Mouvement"]["MobilityActionExtraBits"]
                            .as_i64()
                            .unwrap(),
                },
                keyframe: FactsKeyframeProfile {
                    header_bits: v["ProfilCalibre"]["Cadre"]["EnTeteBits"].as_i64().unwrap(),
                    size_word_bits: v["ProfilCalibre"]["Cadre"]["MotDeTailleBits"]
                        .as_i64()
                        .unwrap(),
                },
                mpp: FactsMppWidths {
                    lead: v["ProfilCalibre"]["MPP"]["Lead"].as_i64().unwrap(),
                    index: v["ProfilCalibre"]["MPP"]["Index"].as_i64().unwrap(),
                },
                grammar: FactsScanGrammar {
                    corruption_check: v["ProfilCalibre"]["Grammaire"]["ControleDeCorruption"]
                        .as_bool()
                        .unwrap(),
                    new_record_tail_bits: v["ProfilCalibre"]["Grammaire"]["BitsDeQueueRecordNew"]
                        .as_i64()
                        .unwrap(),
                    default_state_by_archetype:
                        v["ProfilCalibre"]["Grammaire"]["DeserEtatParArchetype"]
                            .as_bool()
                            .unwrap(),
                    simulation_complete: v["ProfilCalibre"]["Grammaire"]["SimStateComplet"]
                        .as_bool()
                        .unwrap(),
                    baseline_scope: v["ProfilCalibre"]["Grammaire"]["PorteeBaseline"]
                        .as_bool()
                        .unwrap(),
                    writer_absolute: v["ProfilCalibre"]["Grammaire"]["GrammaireEcrivainI0"]
                        .as_bool()
                        .unwrap(),
                    mobility_action_body: v["ProfilCalibre"]["Grammaire"]["CorpsActionMobilite"]
                        .as_bool()
                        .unwrap(),
                    ability_anchor_body: v["ProfilCalibre"]["Grammaire"]["CorpsAncrageCapacite"]
                        .as_bool()
                        .unwrap(),
                    chain_inference: v["ProfilCalibre"]["Grammaire"]["InferenceChaine"]
                        .as_bool()
                        .unwrap(),
                    calibrated_widths: v["ProfilCalibre"]["Grammaire"]["LargeursCalibrees"]
                        .as_object()
                        .map(|m| {
                            m.iter()
                                .map(|(k, v)| (k.as_bytes().to_vec(), v.as_i64().unwrap()))
                                .collect()
                        }),
                    generation_strict: v["ProfilCalibre"]["Grammaire"]["GenerationStricte"]
                        .as_bool()
                        .unwrap(),
                    view_tables: v["ProfilCalibre"]["Grammaire"]["TablesParVue"]
                        .as_bool()
                        .unwrap(),
                    view_classes: v["ProfilCalibre"]["Grammaire"]["ClassesDeVue"]
                        .as_bool()
                        .unwrap(),
                    stub_widths: v["ProfilCalibre"]["Grammaire"]["LargeursBouchon"]
                        .as_object()
                        .map(|m| {
                            m.iter()
                                .map(|(k, v)| (k.as_bytes().to_vec(), v.as_i64().unwrap()))
                                .collect()
                        }),
                },
            },
            bijection_margin: v["BijectionMargin"].as_i64().unwrap(),
            bijection_determined: v["BijectionDetermined"].as_bool().unwrap(),
            probe: if v["Probe"].is_null() {
                None
            } else {
                Some(FactsKillsRelaxedProbe {
                    candidates: v["Probe"]["Candidates"].as_i64().unwrap(),
                    out_of_catalogue: v["Probe"]["OutOfCatalogue"].as_i64().unwrap(),
                    paired: v["Probe"]["Paired"].as_i64().unwrap(),
                    uncovered: v["Probe"]["Uncovered"].as_i64().unwrap(),
                    tags: v["Probe"]["Tags"]
                        .as_array()
                        .map(|a| a.iter().map(|v| v.as_u64().unwrap() as u32).collect()),
                })
            },
        })
    }
}
fn projection(v: &Option<FactsKillsResult>) -> Value {
    json!(v.as_ref().map(|v|json!({"Kills":v.kills.as_ref().map(|a|a.iter().map(|v|json!({"TimeMS":json!(v.time_ms),"Victim":json!(v.victim),"Feed":json!({"Killer":json!(v.feed.killer),"Present":json!(v.feed.present)}),"Source":json!({"Tag":json!(v.source.tag),"Display":json!(v.source.display),"Named":json!(v.source.named),"Class":json!(v.source.class),"Status":json!(v.source.status),"Detail":json!(v.source.detail),"Reserve":json!(v.source.reserve),"Category":json!(v.source.category)}),"Diverges":json!(v.diverges),"Read":json!({"Path":json!(v.read.path),"Origin":json!(v.read.origin),"Multiplicity":json!(v.read.multiplicity)}),"Assist":json!({"Name":json!(v.assist.name),"Index":json!(v.assist.index),"Rejected":json!(v.assist.rejected),"Known":json!(v.assist.known),"Extra":json!(v.assist.extra)}),"KillerDamage":json!({"Pct":json!(v.killer_damage.pct),"Known":json!(v.killer_damage.known)}),"AssistDamage":json!({"Pct":json!(v.assist_damage.pct),"Known":json!(v.assist_damage.known)})})).collect::<Vec<_>>()),"UnclaimedDeaths":v.unclaimed_deaths.as_ref().map(|a|a.iter().map(|v|json!({"TimeMS":json!(v.time_ms),"Victim":json!(v.victim),"VictimXUID":json!(v.victim_xuid),"Source":json!({"Tag":json!(v.source.tag),"Display":json!(v.source.display),"Named":json!(v.source.named),"Class":json!(v.source.class),"Status":json!(v.source.status),"Detail":json!(v.source.detail),"Reserve":json!(v.source.reserve),"Category":json!(v.source.category)}),"Read":json!({"Path":json!(v.read.path),"Origin":json!(v.read.origin),"Multiplicity":json!(v.read.multiplicity)})})).collect::<Vec<_>>()),"Coverage":json!({"Covered":json!(v.coverage.covered),"RealPairs":json!(v.coverage.real_pairs),"ReconstructedPairs":json!(v.coverage.reconstructed_pairs),"GhostPairs":json!(v.coverage.ghost_pairs),"SameInstantPairs":json!(v.coverage.same_instant_pairs),"FeedKills":json!(v.coverage.feed_kills),"FeedDeaths":json!(v.coverage.feed_deaths),"BotDeaths":json!(v.coverage.bot_deaths),"BotKillerDeaths":json!(v.coverage.bot_killer_deaths)}),"Health":json!({"Film":json!(v.health.film),"Candidates":json!(v.health.candidates),"Published":json!(v.health.published),"UnexplainedPair":json!(v.health.unexplained_pair),"UnexplainedSelf":json!(v.health.unexplained_self),"UnexplainedBotIdx":json!(v.health.unexplained_bot_idx),"OutOfRoster":json!(v.health.out_of_roster),"TagOutOfCatalogueWalk":json!(v.health.tag_out_of_catalogue_walk),"TagOutOfCatalogueScan":json!(v.health.tag_out_of_catalogue_scan),"DeathsReal":json!(v.health.deaths_real),"DeathsCovered":json!(v.health.deaths_covered)}),"Stats":json!({"Walk":json!({"Population":json!(v.stats.walk.population),"Matched":json!(v.stats.walk.matched),"Published":json!(v.stats.walk.published)}),"Scan":json!({"Population":json!(v.stats.scan.population),"Matched":json!(v.stats.scan.matched),"Published":json!(v.stats.scan.published)}),"SelfWalk":json!({"Population":json!(v.stats.self_walk.population),"Matched":json!(v.stats.self_walk.matched),"Published":json!(v.stats.self_walk.published)}),"SelfScan":json!({"Population":json!(v.stats.self_scan.population),"Matched":json!(v.stats.self_scan.matched),"Published":json!(v.stats.self_scan.published)}),"Bot":json!({"Population":json!(v.stats.bot.population),"Matched":json!(v.stats.bot.matched),"Published":json!(v.stats.bot.published)}),"BotKiller":json!({"Population":json!(v.stats.bot_killer.population),"Matched":json!(v.stats.bot_killer.matched),"Published":json!(v.stats.bot_killer.published)}),"Unclaimed":json!({"Population":json!(v.stats.unclaimed.population),"Matched":json!(v.stats.unclaimed.matched),"Published":json!(v.stats.unclaimed.published)}),"Redundant":json!(v.stats.redundant),"NoBit":json!(v.stats.no_bit),"Agree":json!(v.stats.agree),"Disagree":json!(v.stats.disagree),"MultiCandidate":json!(v.stats.multi_candidate),"PacketsWithEvents":json!(v.stats.packets_with_events),"PacketsLocated":json!(v.stats.packets_located),"Assist":json!({"KillEvents":json!(v.stats.assist.kill_events),"Attached":json!(v.stats.assist.attached),"Multi":json!(v.stats.assist.multi),"Named":json!(v.stats.assist.named),"NoAssist":json!(v.stats.assist.no_assist),"RejectedSelf":json!(v.stats.assist.rejected_self),"RejectedVictim":json!(v.stats.assist.rejected_victim),"RejectedRoster":json!(v.stats.assist.rejected_roster),"FlagSet":json!(v.stats.assist.flag_set),"AssistMulti":json!(v.stats.assist.assist_multi),"AssistExtraTotal":json!(v.stats.assist.assist_extra_total),"AssistFieldDisagree":json!(v.stats.assist.assist_field_disagree),"KillerPctOver100":json!(v.stats.assist.killer_pct_over100),"AssistPctOver100":json!(v.stats.assist.assist_pct_over100),"ParLaFenetre":json!(v.stats.assist.par_la_fenetre),"Gate15":json!(v.stats.assist.gate15)}),"Couples":json!({"MemeInstant":json!(v.stats.couples.meme_instant),"Lus":json!(v.stats.couples.lus),"Recolles":json!(v.stats.couples.recolles),"Perdus":json!(v.stats.couples.perdus),"VictimesBotLues":json!(v.stats.couples.victimes_bot_lues),"Muet":json!(v.stats.couples.muet),"Ambigu":json!(v.stats.couples.ambigu),"Accord":json!(v.stats.couples.accord),"Contradiction":json!(v.stats.couples.contradiction)}),"Appariement":json!({"Identite":json!(v.stats.appariement.identite),"Fenetre":json!(v.stats.appariement.fenetre),"BotFenetre":json!(v.stats.appariement.bot_fenetre),"NonRevendiqueeFenetre":json!(v.stats.appariement.non_revendiquee_fenetre),"CouplesSansIdentite":json!(v.stats.appariement.couples_sans_identite)})}),"Roster":json!({"Names":v.roster.names.as_ref().map(|a|a.iter().map(|v|json!(v)).collect::<Vec<_>>()),"Humans":json!(v.roster.humans),"Bots":v.roster.bots.as_ref().map(|a|a.iter().map(|v|json!({"Slot":json!(v.slot),"BotID":json!(v.bot_id),"Name":json!(v.name)})).collect::<Vec<_>>()),"IndexToName":v.roster.index_to_name.as_ref().map(|a|a.iter().map(|v|json!(v)).collect::<Vec<_>>()),"UnpinnedBots":v.roster.unpinned_bots.as_ref().map(|a|a.iter().map(|v|json!({"Slot":json!(v.slot),"BotID":json!(v.bot_id),"Name":json!(v.name)})).collect::<Vec<_>>()),"BotsSuccedes":json!(v.roster.bots_succedes),"IndexSource":v.roster.index_source.as_ref().map(|a|a.iter().map(|v|json!(v)).collect::<Vec<_>>()),"FilmTable":json!({"Refusal":json!(v.roster.film_table.refusal),"Build":json!(v.roster.film_table.build),"Seats":json!(v.roster.film_table.seats),"Pinned":json!(v.roster.film_table.pinned),"AddedNames":json!(v.roster.film_table.added_names),"BotConflict":json!(v.roster.film_table.bot_conflict),"DuplicateName":json!(v.roster.film_table.duplicate_name),"OutOfRange":json!(v.roster.film_table.out_of_range),"Inferred":json!(v.roster.film_table.inferred),"FreeNames":json!(v.roster.film_table.free_names),"MotifPinned":json!(v.roster.film_table.motif_pinned),"MotifAgree":json!(v.roster.film_table.motif_agree),"MotifContradict":json!(v.roster.film_table.motif_contradict),"MotifDuplicate":json!(v.roster.film_table.motif_duplicate),"MotifReadings":json!(v.roster.film_table.motif_readings),"MotifDisagreements":json!(v.roster.film_table.motif_disagreements),"MotifAbsent":json!(v.roster.film_table.motif_absent),"Agree":json!(v.roster.film_table.agree),"Contradict":json!(v.roster.film_table.contradict),"Silent":json!(v.roster.film_table.silent)})}),"Calibration":json!(v.calibration),"ProfilCalibre":json!({"Mouvement":json!({"Traversal":json!({"IndexW":json!(v.profil_calibre.movement.traversal.index_width),"AxisW":v.profil_calibre.movement.traversal.axis_widths.iter().map(|v|json!(v)).collect::<Vec<_>>(),"Region":json!(v.profil_calibre.movement.traversal.region)}),"WorldObject":json!({"IndexW":json!(v.profil_calibre.movement.world_object.index_width),"AxisW":v.profil_calibre.movement.world_object.axis_widths.iter().map(|v|json!(v)).collect::<Vec<_>>(),"Region":json!(v.profil_calibre.movement.world_object.region)}),"DeltaQuantum":json!(v.profil_calibre.movement.delta_quantum.to_bits()),"DeltaAxisWidth":json!(v.profil_calibre.movement.delta_axis_width),"Range":v.profil_calibre.movement.range.iter().map(|v|json!({"Min":json!(v.min.to_bits()),"Max":json!(v.max.to_bits())})).collect::<Vec<_>>(),"FullPrecision":json!(v.profil_calibre.movement.full_precision),"DeltaHasHandleTail":json!(v.profil_calibre.movement.delta_has_handle_tail),"CalibratedSkip":json!(v.profil_calibre.movement.calibrated_skip),"MobilityActionExtraBits":json!(v.profil_calibre.movement.mobility_action_extra_bits)}),"Cadre":json!({"EnTeteBits":json!(v.profil_calibre.keyframe.header_bits),"MotDeTailleBits":json!(v.profil_calibre.keyframe.size_word_bits)}),"MPP":json!({"Lead":json!(v.profil_calibre.mpp.lead),"Index":json!(v.profil_calibre.mpp.index)}),"Grammaire":json!({"ControleDeCorruption":json!(v.profil_calibre.grammar.corruption_check),"BitsDeQueueRecordNew":json!(v.profil_calibre.grammar.new_record_tail_bits),"DeserEtatParArchetype":json!(v.profil_calibre.grammar.default_state_by_archetype),"SimStateComplet":json!(v.profil_calibre.grammar.simulation_complete),"PorteeBaseline":json!(v.profil_calibre.grammar.baseline_scope),"GrammaireEcrivainI0":json!(v.profil_calibre.grammar.writer_absolute),"CorpsActionMobilite":json!(v.profil_calibre.grammar.mobility_action_body),"CorpsAncrageCapacite":json!(v.profil_calibre.grammar.ability_anchor_body),"InferenceChaine":json!(v.profil_calibre.grammar.chain_inference),"LargeursCalibrees":v.profil_calibre.grammar.calibrated_widths.as_ref().map(|m|m.iter().map(|(k,v)|(String::from_utf8(k.clone()).unwrap(),json!(v))).collect::<serde_json::Map<_,_>>()),"GenerationStricte":json!(v.profil_calibre.grammar.generation_strict),"TablesParVue":json!(v.profil_calibre.grammar.view_tables),"ClassesDeVue":json!(v.profil_calibre.grammar.view_classes),"LargeursBouchon":v.profil_calibre.grammar.stub_widths.as_ref().map(|m|m.iter().map(|(k,v)|(String::from_utf8(k.clone()).unwrap(),json!(v))).collect::<serde_json::Map<_,_>>())})}),"BijectionMargin":json!(v.bijection_margin),"BijectionDetermined":json!(v.bijection_determined),"Probe":v.probe.as_ref().map(|v|json!({"Candidates":json!(v.candidates),"OutOfCatalogue":json!(v.out_of_catalogue),"Paired":json!(v.paired),"Uncovered":json!(v.uncovered),"Tags":v.tags.as_ref().map(|a|a.iter().map(|v|json!(v)).collect::<Vec<_>>())}))})))
}
#[test]
fn native_facts_kills_json() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-kills-json-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let f: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(f["writers"].as_array().unwrap().len(), 163);
    assert_eq!(f["reads"].as_array().unwrap().len(), 9685);
    for (i, row) in f["writers"].as_array().unwrap().iter().enumerate() {
        let v = source(&row["source"]);
        let got = encode_facts_kills_json(v.as_ref());
        if row["error"] == "" {
            assert_eq!(
                got.unwrap(),
                unhex(row["encoded"].as_str().unwrap()),
                "writer {i}"
            );
        } else {
            assert_eq!(
                got.unwrap_err(),
                row["error"].as_str().unwrap(),
                "writer error {i}"
            );
        }
    }
    for (i, row) in f["reads"].as_array().unwrap().iter().enumerate() {
        let mut reader = FactsKillsJsonReader::default();
        for (j, step) in row["steps"].as_array().unwrap().iter().enumerate() {
            let bytes = unhex(step["input"].as_str().unwrap());
            match reader.read(&bytes) {
                Ok(v) => {
                    assert_eq!(step["error"], "", "success {i}/{j}");
                    assert_eq!(projection(&v), step["decoded"], "read {i}/{j}");
                }
                Err(e) => {
                    assert_eq!(
                        e,
                        step["error"].as_str().unwrap(),
                        "error {i}/{j}: {}",
                        String::from_utf8_lossy(&bytes)
                    );
                    assert!(reader.read(b"{}").is_err());
                    break;
                }
            }
        }
    }
}
