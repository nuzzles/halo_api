//! Known Halo Infinite map calibrations, independent of individual map variants.
//!
//! Bounds: LevelUp `data/titles/halo_infinite/reference/map_quant_bounds.json`,
//! blob 877e3cbcd324b983c76fed2bac409710cb31e67d at commit
//! 413971bcc485be53a2fa8cf229e170539a5146f8. All 20 developer-map entries
//! are included. The 81 Forge-canvas entries share one calibration.
//! House of Reckoning (`pve_house`) has no bounds in that source and is unsupported.

use super::{MapDefinition, PositionBounds};
use uuid::Uuid;

/// Source of the coordinate calibration and verified asset-to-map mapping.
pub const SOURCE: &str = "LevelUp map_quant_bounds.json blob 877e3cbcd324b983c76fed2bac409710cb31e67d; asset modules from inventaire_rotation_ugc_2026-08-27.json at 413971bcc485be53a2fa8cf229e170539a5146f8 and the verified Ranked Arena/Octagon fixtures";

macro_rules! define_known_maps {
    ($($variant:ident => $constant:ident, $name:literal, $region:literal, $bits:literal,
       $min:expr, $max:expr;)*) => {
        /// A developer map or the shared Forge-canvas coordinate space.
        /// Forge edits of a developer map retain that developer map's calibration.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum KnownMap { $(#[doc = $name] $variant,)* }

        $(#[doc = $name]
        pub const $constant: MapDefinition = MapDefinition {
            map: KnownMap::$variant,
            name: $name,
            bounds: PositionBounds { region: $region, min: $min, max: $max },
            region_index_bits: $bits,
            provenance: SOURCE,
        };)*

        /// Every supported calibration, with a single entry for Forge canvases.
        pub const ALL: &[MapDefinition] = &[$($constant,)*];

        impl KnownMap {
            /// Return this map's shared coordinate calibration.
            pub const fn definition(self) -> MapDefinition {
                match self { $(Self::$variant => $constant,)* }
            }
        }
    };
}

define_known_maps! {
    Aquarius => AQUARIUS, "Aquarius", 0, 1,
        [-39.014282, -27.861597, -2.8262208], [38.79729, 18.353075, 15.290748];
    Bazaar => BAZAAR, "Bazaar", 0, 1,
        [-973.866, -361.43918, -86.55155], [179.37695, 1047.0077, 489.09164];
    Behemoth => BEHEMOTH, "Behemoth", 0, 1,
        [-621.9919, -664.6657, -123.74587], [510.3757, 566.11115, 376.25412];
    Breaker => BREAKER, "Breaker", 0, 1,
        [-50.430653, -59.508266, -27.030104], [46.284206, 72.88043, 30.20305];
    Catalyst => CATALYST, "Catalyst", 0, 1,
        [-250.0, -250.0, -250.0], [47.489002, 158.4008, 153.23];
    Chasm => CHASM, "Chasm", 0, 1,
        [-1233.902, -1130.1733, -250.0], [1233.902, 1130.1733, 1015.4839];
    Cliffhanger => CLIFFHANGER, "Cliffhanger", 0, 1,
        [-41.102932, -56.606728, -84.37054], [72.109375, 57.211975, 53.179653];
    Deadlock => DEADLOCK, "Deadlock", 0, 1,
        [-250.0, -250.0, -250.0], [250.0, 250.0, 250.0];
    Forbidden => FORBIDDEN, "Forbidden", 0, 1,
        [-41.11919, -32.20354, -250.0], [41.348022, 30.58435, 18.519018];
    Forest => FOREST, "Forest", 0, 1,
        [-59.680088, -42.27246, -59.00115], [25.232569, 51.440674, 18.694605];
    Fragmentation => FRAGMENTATION, "Fragmentation", 0, 1,
        [-1136.2743, -702.29663, -250.0], [543.44055, 784.0803, 250.0];
    Highpower => HIGHPOWER, "Highpower", 0, 1,
        [-1753.5928, -2474.1824, -901.7554], [2287.1992, 3033.661, 901.7554];
    Illusion => ILLUSION, "Illusion", 0, 1,
        [-1233.902, -1130.1733, -250.0], [1233.902, 1130.1733, 1015.4839];
    LaunchSite => LAUNCH_SITE, "Launch Site", 0, 1,
        [-631.04297, -642.97345, -53.24063], [631.04297, 642.97345, 446.75937];
    LiveFire => LIVE_FIRE, "Live Fire", 1, 2,
        [-16.719454, -10.102863, -9.330649], [46.513535, 53.67172, 13.5679];
    Oasis => OASIS, "Oasis", 0, 1,
        [-191.43637, -151.62828, -78.3619], [146.81325, 140.49544, 112.80506];
    Prism => PRISM, "Prism", 0, 1,
        [-114.45093, -27.205103, -250.0], [28.68396, 68.92768, 61.703808];
    Recharge => RECHARGE, "Recharge", 0, 1,
        [-993.9214, -1320.3536, -250.0], [1713.4697, 1320.3536, 250.0];
    Scarr => SCARR, "Scarr", 0, 1,
        [-70.57142, -48.46288, -250.0], [68.996315, 41.17087, 63.835415];
    Streets => STREETS, "Streets", 0, 1,
        [-24.32224, -23.018236, -51.375183], [27.407486, 29.866623, 16.546118];
    Forge => FORGE, "Forge", 0, 1,
        [-231.00145, -227.08232, -946.215], [231.63503, 226.34886, 242.32658];
}

impl KnownMap {
    /// Classify an asset in the built-in catalogue. Custom glossary registrations
    /// are resolved through [`super::MapGlossary`]. Unknown IDs never imply Forge.
    pub fn from_asset_id(asset_id: Uuid) -> Option<Self> {
        ASSETS
            .iter()
            .find(|(id, _, _)| *id == asset_id)
            .map(|(_, _, map)| *map)
    }
}

// UUIDs are checked at compile time. Names belong to assets: multiple differently
// named Forge maps or edits of a developer map share one KnownMap definition.
macro_rules! map_assets {
    ($($id:literal => $map:ident, $name:literal;)*) => {
        pub(super) const ASSETS: &[(Uuid, &str, KnownMap)] = &[
            $((uuid::uuid!($id), $name, KnownMap::$map),)*
        ];
    };
}

// Module/canvas identity comes from LevelUp's UGC inventory at the commit above,
// not from guessing based on asset names. Its explicit canvas/level-ID pairs also
// identify the Argyle and Detachment entries that only retain a level ID.
// The captured Ranked Arena rotation and Octagon retain their verified fixture mappings.
map_assets! {
    "33c0766c-ef15-48f8-b298-34aba5bff3b4" => Aquarius, "Aquarius";
    "667072ae-ba00-4414-adda-8203e8c49295" => Aquarius, "Aquarius";
    "298d5036-cd43-47b3-a4bd-31e127566593" => Bazaar, "Bazaar";
    "53136ad9-0fd6-4271-8752-31d114b9561e" => Behemoth, "Behemoth";
    "e6cbfe01-665b-4a8c-bf3a-d63a65a7c890" => Breaker, "Breaker";
    "e8567de9-6e15-458d-b2cf-7762bcbaa6f2" => Breaker, "Breaker Firefight";
    "11ca6a12-da14-4315-94e4-130e1a5e0848" => Catalyst, "Catalyst";
    "4b6a9a35-520d-4784-86fd-5403a1d2fd8a" => Catalyst, "Catalyst";
    "74b8c681-a3b0-422b-9fea-c111d3c979da" => Catalyst, "Catalyst";
    "e859cf75-9b8a-429a-91be-2376681c8537" => Catalyst, "Catalyst";
    "fc1ced39-128b-439d-9b44-4710225090f3" => Chasm, "Chasm";
    "81274d6f-6a94-425a-a16e-3bdb1e2eea9d" => Cliffhanger, "Cliffhanger";
    "f1c0aa1c-5cfb-4669-962a-91d81a683050" => Cliffhanger, "Cliffhanger Firefight";
    "08607bf4-6abe-4a5b-9547-290a6cc1433e" => Deadlock, "Deadlock";
    "6f82df05-3b24-4746-8579-e0c5ef9e9d64" => Deadlock, "Deadlock";
    "372ce070-302d-4de8-a633-eadf488aee1f" => Deadlock, "Deadlock Firefight";
    "7e8b3a2c-459d-4f33-9066-66041b5a36c4" => Forbidden, "Forbidden";
    "db2b5f9c-9895-4f73-9d57-44f9b191f3cf" => Forbidden, "Forbidden";
    "ea51a3dd-2125-4e5b-872d-25f1835edd29" => Forbidden, "Forbidden";
    "5c215079-c34d-4d25-b86a-435e35edf188" => Forbidden, "Forbidden - Ranked";
    "619bea21-f1e6-461f-8a7d-2bb4f905d0ca" => Forest, "Forest";
    "78da545f-a168-4a5e-9c8d-dd379067c352" => Forge, "Absolution";
    "023507d1-7566-458d-b3d7-31627c40c01d" => Forge, "Apostle";
    "0b22acc2-fdc2-4c80-b59c-dc84bded06d8" => Forge, "Aqueduct";
    "2e35d8c0-8292-465c-8d69-35b5ff8b70db" => Forge, "Argyle";
    "78677080-db7a-429e-84fe-f041d8342c37" => Forge, "Argyle";
    "dd600260-d91c-4d77-9990-3f35873c90a1" => Forge, "Argyle";
    "beaf8ac9-fa85-4562-8def-28d9428dbddb" => Forge, "Arrival";
    "e0bd2b53-30d6-4a20-8664-5905ac5748d1" => Forge, "Arrival";
    "9ad226d8-8947-4c5b-95bc-d220187698c1" => Forge, "Banished Narrows";
    "c9d416e2-ca9d-406d-b1fb-3da688e84a2f" => Forge, "Cliffside";
    "8be179f7-8940-4868-b881-44cad1ca8711" => Forge, "Corpo";
    "bae4df14-4f4a-424c-aac1-2f795c807146" => Forge, "Critical Dewpoint";
    "f718abc4-1bf6-4665-8050-5e20071e04a4" => Forge, "Critical Dewpoint";
    "63d634be-0319-489d-8c21-9c4e012f664f" => Forge, "Curfew";
    "26874260-cb7a-4e59-942e-55b1d24fd8d0" => Forge, "Daimyo";
    "73966558-e9a4-4533-bd5b-862a22ac03ce" => Forge, "Dead Water";
    "d39600e2-3c35-4a3a-bdf5-7b3cbdde98e1" => Forge, "Detachment";
    "50a1a3b5-c54d-4e40-8744-b92051d48897" => Forge, "Diminished";
    "921aebb1-783d-45e4-bacd-7ad869fa8dae" => Forge, "Domicile";
    "f631b010-3eed-4b84-8e09-4ed90ab32c26" => Forge, "Domicile";
    "e4bb06db-065f-4902-b93b-d8dac315eac4" => Forge, "Dredge";
    "90cd321d-1439-49de-943d-06245a407909" => Forge, "Dynasty";
    "70dd38c5-2eb7-4db3-8901-0dfca292ff18" => Forge, "Empyrean";
    "d035fc3e-f298-4c14-9487-465be2e1dc1f" => Forge, "Empyrean";
    "78905252-4e2f-4cf7-840f-c0bad5a6b67f" => Forge, "Exiled";
    "9bb8d9df-ff6c-4a6e-b151-436bbb2c0907" => Forge, "Exiled";
    "f32b61f8-b94d-4a1b-9c69-f9d1c47b8e9b" => Forge, "Exiled Firefight";
    "1ede38fa-4d30-4dfa-a8b7-5d08bf4e46e3" => Forge, "Fortitude";
    "a54808fb-9bf5-432a-a3c3-f76cbea944c1" => Forge, "Fortress";
    "a6b56a60-371e-41d4-a248-0df797cb3a16" => Forge, "Goliath";
    "3d52899a-2a54-49ba-94f7-583eea8be9c8" => Forge, "High Ground";
    "d5c5eb4f-0dcb-4677-a866-eae0dcbfde9b" => Forge, "Insolence";
    "654dff62-d618-496a-8914-06ab73d991e3" => Forge, "Interference";
    "01af558d-53ab-4f05-ba68-92d805fc6260" => Forge, "Isolation";
    "5dbe246f-7c75-4b60-a5cf-1b4f26e94734" => Forge, "Isolation";
    "98a83f87-2420-48c6-93f8-cc6b62d73235" => Forge, "Kaiketsu";
    "df7dbf08-b8de-4ade-9d7f-1947128c9ae4" => Forge, "Kiken'na";
    "9f589ad8-625d-49b8-9a57-1e560576a4b9" => Forge, "Kusini Bay";
    "b10f80d2-77bc-45e3-bdd6-f4d2fc22180e" => Forge, "Kusini Bay Firefight";
    "3c1f2523-0515-4b42-b651-33b0a01397c3" => Forge, "Last Broadcast";
    "1a6cfc2e-ec86-48e1-9464-1ce1bff6ed48" => Forge, "Lattice";
    "a289bafe-102e-4363-98f7-80b596007338" => Forge, "Obituary";
    "6ece0a92-ef00-459c-b882-cea608cafe60" => Forge, "Octagon (fo09_academy)";
    "255bbe78-b191-476e-b0ae-0763c3bc2f44" => Forge, "Opulence";
    "46a8319c-2c63-46ee-9382-788906dcb049" => Forge, "Origin";
    "6f24843d-a636-4d7d-b4bd-b077a12e2a3b" => Forge, "Perilous";
    "133c0185-24ed-4bc2-b834-62db5c936257" => Forge, "Rat's Nest";
    "97696a83-ac78-407d-85b1-ef74d3c64c4a" => Forge, "Rat's Nest";
    "41217472-3020-4bd8-bce9-b2a2b0d50896" => Forge, "Refuge";
    "cd08bc7a-7ba5-4502-be87-c58b641fc94d" => Forge, "Salvation";
    "2962c4e0-ab15-4979-8d28-c0792632c862" => Forge, "Shiro";
    "98783453-ce40-4020-9e87-62099a290b62" => Forge, "Smallhalla";
    "4a5e5612-2b2e-4375-a0b3-9335a68815f3" => Forge, "Solitude";
    "f1cc3b4e-471c-4ec5-b855-1db7d9e6ce42" => Forge, "Solitude";
    "ee43d273-8677-45c2-a8cd-aedd2c463dc9" => Forge, "Solution";
    "7a9265af-a880-487b-8829-68d88fcfb145" => Forge, "Starboard";
    "3c93772b-0338-4ee1-8108-4c0bb52ee219" => Forge, "Suban";
    "95b69e4b-485f-4c6c-9b00-4bd68c94c1e9" => Forge, "Sylvanus";
    "edcd4467-6846-455f-ac44-f1034476f774" => Forge, "Takamanohara";
    "ec1d4f25-8e4a-4cf7-ae48-43fa9e1a3808" => Forge, "The Pit";
    "27c4e8a1-d982-4e74-a1ec-d32f45551194" => Forge, "The Pit - Ranked";
    "28a3ac28-f69d-4fa9-9ebf-a0449c89c8da" => Forge, "Thunderhead";
    "6a1e8432-88ae-4430-8f7d-9ffefc97cc8d" => Forge, "Vacancy";
    "105f5d84-8de1-4908-af3a-1c4f3bf9d642" => Forge, "Vagabond";
    "688c2033-35de-461e-9394-a32c665c964c" => Forge, "Vallaheim";
    "8c95653b-e3ee-406c-ac4f-64f54a732708" => Forge, "Vallaheim";
    "e8268e75-6583-42ad-9e0c-2d2f043f5f0f" => Forge, "Vallaheim Firefight";
    "0661af5e-8b6d-44c0-bb7c-9c76cdcc624d" => Forge, "Waterworks";
    "96783263-5c9c-4dc4-a2ad-d1dfc35883b2" => Forge, "Waterworks";
    "068f41f4-e3dd-4bec-b297-d2ded85ab54b" => Fragmentation, "Fragmentation";
    "4f196016-0101-4844-8358-2504f7c44656" => Fragmentation, "Fragmentation";
    "ad2d5fe1-033d-4168-b91b-5d96595d6644" => Fragmentation, "Fragmentation Firefight";
    "33c6505d-1cfa-43c6-9fa9-311eb0502ad9" => Highpower, "Highpower";
    "c494ef7c-d203-42a9-9c0f-b3f576334501" => Highpower, "Highpower";
    "86ef3b1c-2f39-4c29-8c19-65ab84a704c2" => Illusion, "Illusion";
    "56a11b8c-64d1-4537-8893-a9241e4d5b93" => LaunchSite, "Launch Site";
    "309253f8-7a75-48ff-83e1-e7fb3db2ac47" => LiveFire, "Live Fire";
    "b6aca0c7-8ba7-4066-bf91-693571374c3c" => LiveFire, "Live Fire";
    "6aa0a116-66a6-4242-a1b3-41aa417d6dc6" => Oasis, "Oasis";
    "f566aa62-afe4-4f33-b44f-7a314cb23f9b" => Oasis, "Oasis Firefight";
    "92d23264-d3b9-462e-adbc-8ddb44e81966" => Prism, "Prism";
    "336b5174-3579-4fd8-b2f0-922e4a5f7628" => Recharge, "Recharge";
    "3adbac00-12bc-48c8-9c4c-292b869ebdf6" => Recharge, "Recharge";
    "8420410b-044d-44d7-80b6-98a766c8c39f" => Recharge, "Recharge";
    "247637f8-1ed2-47de-8ff0-fd4b68f50888" => Scarr, "Scarr";
    "5c2898ac-bf38-48c9-8c46-7a4c09f3cd1d" => Scarr, "Scarr Firefight";
    "e23ea388-9bcb-4180-a0dc-fbe987751b9e" => Streets, "Streets";
    "f0a1760f-0d4a-4bcc-ac7a-e8f9aee331dc" => Streets, "Streets";
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn developer_maps_and_forge_match_levelup_axis_widths() {
        // Independent axisWidths published in the pinned map_quant_bounds.json.
        let expected = [
            (KnownMap::Aquarius, [13, 12, 11]),
            (KnownMap::Bazaar, [17, 17, 16]),
            (KnownMap::Behemoth, [17, 17, 15]),
            (KnownMap::Breaker, [13, 13, 12]),
            (KnownMap::Catalyst, [15, 15, 15]),
            (KnownMap::Chasm, [18, 18, 17]),
            (KnownMap::Cliffhanger, [13, 13, 14]),
            (KnownMap::Deadlock, [15, 15, 15]),
            (KnownMap::Forbidden, [13, 12, 14]),
            (KnownMap::Forest, [13, 13, 13]),
            (KnownMap::Fragmentation, [17, 17, 15]),
            (KnownMap::Highpower, [18, 19, 17]),
            (KnownMap::Illusion, [18, 18, 17]),
            (KnownMap::LaunchSite, [17, 17, 15]),
            (KnownMap::LiveFire, [12, 12, 11]),
            (KnownMap::Oasis, [15, 15, 14]),
            (KnownMap::Prism, [14, 13, 15]),
            (KnownMap::Recharge, [18, 18, 15]),
            (KnownMap::Scarr, [14, 13, 15]),
            (KnownMap::Streets, [12, 12, 12]),
            (KnownMap::Forge, [15, 15, 17]),
        ];
        assert_eq!(ALL.len(), 21);
        assert_eq!(ALL.len(), expected.len());
        for (map, widths) in expected {
            assert_eq!(map.definition().bounds.axis_bits(), Some(widths));
            assert!(ASSETS.iter().any(|&(_, _, known)| known == map));
        }
        assert_eq!(
            ALL.iter().map(|m| m.map).collect::<HashSet<_>>().len(),
            ALL.len()
        );
        assert_eq!(
            ASSETS
                .iter()
                .map(|&(id, _, _)| id)
                .collect::<HashSet<_>>()
                .len(),
            ASSETS.len()
        );
        for &(id, _, map) in ASSETS {
            assert_eq!(KnownMap::from_asset_id(id), Some(map));
        }
    }

    #[test]
    fn forge_canvases_share_one_definition_but_native_edits_keep_their_bounds() {
        for id in [
            uuid::uuid!("46a8319c-2c63-46ee-9382-788906dcb049"), // Origin
            uuid::uuid!("78677080-db7a-429e-84fe-f041d8342c37"), // Argyle
            uuid::uuid!("6ece0a92-ef00-459c-b882-cea608cafe60"), // Octagon
        ] {
            assert_eq!(KnownMap::from_asset_id(id), Some(KnownMap::Forge));
        }
        assert_eq!(
            KnownMap::from_asset_id(uuid::uuid!("667072ae-ba00-4414-adda-8203e8c49295")),
            Some(KnownMap::Aquarius),
        );
        assert_eq!(
            KnownMap::from_asset_id(uuid::uuid!("309253f8-7a75-48ff-83e1-e7fb3db2ac47")),
            Some(KnownMap::LiveFire),
        );
        assert_ne!(AQUARIUS.bounds, FORGE.bounds);
        assert_eq!(LIVE_FIRE.bounds.region, 1);
        assert_eq!(LIVE_FIRE.region_index_bits, 2);
        assert_eq!(KnownMap::from_asset_id(Uuid::nil()), None);
        // LevelUp lists House of Reckoning but publishes no quantization bounds.
        assert_eq!(
            KnownMap::from_asset_id(uuid::uuid!("eaf608f0-6ec3-444f-a51a-9c1de5d0bf5c")),
            None,
        );
    }
}
