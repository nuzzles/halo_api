//! Loaded native grenade signatures, build selection, and coverage diagnostics.
use super::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrenadePreamble {
    pub bits: usize,
    pub default_state: u32,
    pub author_bit: usize,
}
impl Default for GrenadePreamble {
    fn default() -> Self {
        Self {
            bits: 24,
            default_state: 0x40c00,
            author_bit: 103,
        }
    }
}
impl GrenadePreamble {
    /// Build metadata still governs a v41-labelled source. Other source major
    /// versions are rejected by NativeFilmContext's profile resolver.
    fn for_v41_build(build: &str) -> (Self, bool) {
        match build {
            "HI_1_12_0" | "HI_1_13_0" => (Self::default(), true),
            "HI_1_8_0" | "HI_1_9_0" | "HI_1_10_0" | "HI_1_11_0" => (
                Self {
                    bits: 23,
                    default_state: 0x20600,
                    author_bit: 100,
                },
                true,
            ),
            "HI_1_4_1" => (
                Self {
                    bits: 23,
                    default_state: 0x20600,
                    author_bit: 99,
                },
                true,
            ),
            _ => (Self::default(), false),
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum ContextGrenadeError {
    #[error("aucun chunk film lisible")]
    NoReadableChunks,
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextGrenadePacket {
    pub chunk: i64,
    pub source: FilmPacket,
    pub packet_index: usize,
    pub scan: GrenadeThrowStream,
}
#[derive(Debug)]
pub struct ContextGrenadeScan {
    pub grammar: GrenadeGrammar,
    pub preamble: GrenadePreamble,
    /// Registry failure selects the native fallback but remains inspectable.
    pub registry_error: Option<NativeContextRegistryError>,
    /// Native nil when no throw was published, including a successful empty scan.
    pub reads: Option<Vec<FilmGrenadeThrow>>,
    pub stats: GrenadeThrowStats,
    pub chunks: usize,
    pub packets: Vec<ContextGrenadePacket>,
    pub diagnostics: Vec<StatborgDiagnostic>,
}
/// ScanGrenadeThrows with native numbered traversal and post-scan diagnostics.
/// Diagnostics are delivered at their native boundary to the embedding sink.
pub fn scan_context_grenade_throws(
    context: &NativeFilmContext<'_>,
    mut observe: impl FnMut(&StatborgDiagnostic),
) -> (ContextGrenadeScan, Option<ContextGrenadeError>) {
    let mut out = ContextGrenadeScan {
        grammar: v41_grenade_grammar(None, None),
        preamble: GrenadePreamble::default(),
        registry_error: None,
        reads: None,
        stats: GrenadeThrowStats::default(),
        chunks: 0,
        packets: Vec::new(),
        diagnostics: Vec::new(),
    };
    let error=(||->Result<(),ContextGrenadeError>{
        let profile=context.profile()?;
        let (preamble,known)=GrenadePreamble::for_v41_build(profile.build());
        out.preamble=preamble;
        match context.registry() {
            Ok(reg)=>out.grammar=v41_grenade_grammar(Some(&reg.registry),None),
            Err(e)=>out.registry_error=Some(*e),
        }
        out.grammar.known_build=known;
        for &chunk in context.chunk_numbers(){
            let Some((data,packets))=context.chunk_at(chunk) else {continue};
            out.chunks+=1;
            for (packet_index,source) in packets.iter().copied().enumerate(){
                if source.packet_type!=0{continue;}
                let payload=&data[source.payload_offset..source.payload_offset+source.payload_size];
                let mut scan=grenade_throws::decode_grenade_throws_with_preamble(payload,out.grammar.clone(),preamble);
                out.stats.patterns+=scan.stats.patterns;out.stats.other_archetypes+=scan.stats.other_archetypes;out.stats.rejected_known_ids+=scan.stats.rejected_known_ids;out.stats.indeterminate_archetype+=scan.stats.indeterminate_archetype;
                for r in &mut scan.records {r.timestamp_us=source.timestamp_us;r.chunk=chunk;r.packet_index=packet_index;out.reads.get_or_insert_with(Vec::new).push(r.clone());}
                out.packets.push(ContextGrenadePacket{chunk,source,packet_index,scan});
            }
        }
        if out.chunks==0{return Err(ContextGrenadeError::NoReadableChunks);}
        out.stats.published=out.reads.as_ref().map_or(0,Vec::len);
        if !known {out.diagnostics.push(StatborgDiagnostic {level:"WARN".into(),message:"lancers de grenade : cle du film ABSENTE de la table d amorce — profil de reference applique (repli_amorce_grenade_profil_de_reference)".into(),attributes:vec![("amorce_bits".into(),json!(preamble.bits)),("type_index".into(),json!(out.grammar.projectile_archetype))]});}
        if !out.grammar.resolved_by_name{out.diagnostics.push(StatborgDiagnostic {level:"WARN".into(),message:"lancers de grenade : archetype projectile NON RESOLU par le nom de ses composants — archetype de reference applique".into(),attributes:vec![("type_index".into(),json!(out.grammar.projectile_archetype))]});}
        let s=&out.stats;
        out.diagnostics.push(StatborgDiagnostic {level:"INFO".into(),message:"lancers de grenade : couverture du balayage".into(),attributes:vec![("amorceBits".into(),json!(preamble.bits)),("indexAuteurBit".into(),json!(preamble.author_bit)),("typeIndexProjectile".into(),json!(out.grammar.projectile_archetype)),("motifs".into(),json!(s.patterns)),("naissancesAutresArchetypes".into(),json!(s.other_archetypes)),("naissancesAutresArchetypesAvecIdentifiant".into(),json!(s.rejected_known_ids)),("naissancesTypeIndexIndetermine".into(),json!(s.indeterminate_archetype)),("lancersPublies".into(),json!(s.published))]});
        for d in &out.diagnostics{observe(d);}
        Ok(())
    })().err();
    (out, error)
}
