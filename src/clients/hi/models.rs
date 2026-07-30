use std::{borrow::Cow, collections::BTreeMap};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};

const WAYPOINT_FILE_BASE_URL: &str = "https://gamecms-hacs.svc.halowaypoint.com/hi/Waypoint/file";

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Option::<T>::deserialize(deserializer).map(Option::unwrap_or_default)
}

/// A Halo Infinite matchmaking playlist asset ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlaylistId(Cow<'static, str>);

impl PlaylistId {
    pub const BIG_TEAM_BATTLE: Self = Self(Cow::Borrowed("2825d417-93e6-4366-98f9-839a2dc41fe4"));
    pub const RANKED_ARENA: Self = Self(Cow::Borrowed("edfef3ac-9cbe-4fa2-b949-8f29deafd483"));
    pub const RANKED_DOUBLES: Self = Self(Cow::Borrowed("fa5aa2a3-2428-4912-a023-e1eeea7b877c"));
    pub const RANKED_SLAYER: Self = Self(Cow::Borrowed("dcb2e24e-05fb-4390-8076-32a0cdb4326e"));
    pub const SQUAD_BATTLE: Self = Self(Cow::Borrowed("f5580605-660c-43f9-ac69-4075c4a05c5d"));
    pub const TEAM_DOUBLES: Self = Self(Cow::Borrowed("7323be09-2523-47c0-9e0d-64af9534ee22"));

    /// Creates an ID for a playlist that is not included among the named constants.
    pub fn new(value: impl Into<String>) -> Self {
        Self(Cow::Owned(value.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for PlaylistId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for PlaylistId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A Halo Infinite UGC game-mode asset ID paired with an immutable version ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameModeId {
    asset_id: Cow<'static, str>,
    version_id: Cow<'static, str>,
}

impl GameModeId {
    pub const ASSAULT_MULTI_BOMB_BTB: Self = Self::from_static(
        "f7704cb1-476a-447d-ac53-847874a9dae0",
        "94d1b9c3-b7fa-48a2-ac9a-64bed263922a",
    );
    pub const ASSAULT_NEUTRAL_BOMB_SQUAD: Self = Self::from_static(
        "ce9c98fa-e376-4958-a470-111e0d070f70",
        "88f0d4bd-08ad-4475-b0bf-5e222f8a4f30",
    );
    pub const ASSAULT_ONE_BOMB_BTB: Self = Self::from_static(
        "50d8dad3-f49c-4c69-9ced-be4de6b2e844",
        "47a5c157-01d6-4818-a5ed-39b94135edf7",
    );
    pub const ASSAULT_ONE_BOMB_SQUAD: Self = Self::from_static(
        "a34be7a1-0c04-4a5a-8cd4-bb65e76e2b4b",
        "0cd1c5c2-5fee-408b-bfd0-963a5c0496d5",
    );
    pub const BTB_ONE_FLAG_CTF: Self = Self::from_static(
        "b5b791c6-46b9-4b0d-8590-da0d22287e01",
        "4fcc56ac-91c7-458d-8372-44c6159e563e",
    );
    pub const BTB_SENTRY_DEFENSE: Self = Self::from_static(
        "b41b6dab-18b6-4784-a079-dee748fe473b",
        "5e5783a1-a005-4937-a5e8-57cee603cb8d",
    );
    pub const CASTLE_WARS: Self = Self::from_static(
        "823cc0c6-a368-467a-9481-d9de35e7e666",
        "b83ef4ce-eb6d-4d04-af32-69b6acd4c60f",
    );
    pub const CTF_BTB: Self = Self::from_static(
        "1519c0cb-759d-424e-a68e-b9cb870b1e14",
        "030a4f4a-0493-42ae-a831-352172fed8ee",
    );
    pub const CTF_BTB_FIESTA: Self = Self::from_static(
        "72578bb7-547a-4627-b21f-c93cc3213dca",
        "ff344ee6-82e9-4df1-8cfa-0f486d3bce26",
    );
    pub const CTF_BTB_HEAVIES: Self = Self::from_static(
        "3c038d8d-b4b4-4c97-ab53-1e9d3b43d502",
        "f7fb19e5-c283-44d1-b6fe-9fa5accb35ce",
    );
    pub const CTF_DOUBLES: Self = Self::from_static(
        "7be47523-cadc-4fba-8ba9-155ba45e1f59",
        "93156ae1-306d-4d09-9969-eaee46475f57",
    );
    pub const EXTRACTION_BTB: Self = Self::from_static(
        "51502806-5e18-442b-bd85-ea8cb3b6e726",
        "2f83581c-c50d-46b1-920e-c967de2355f7",
    );
    pub const FIESTA_SLAYER: Self = Self::from_static(
        "aca7bbf8-7a18-4aae-8785-1bd3f58275fd",
        "3685f6b2-2860-4e98-9d13-513087edb465",
    );
    pub const INVASION: Self = Self::from_static(
        "0d192b69-6899-4c3c-b63d-4de7beb07f76",
        "ec59fb10-30ee-47e7-a27b-02caf21ac2d9",
    );
    pub const KOTH_DOUBLES: Self = Self::from_static(
        "cbfaa7fe-5963-4762-b655-a99824ba6fdf",
        "408da051-5d42-49fa-8417-5f9a78cf8467",
    );
    pub const RANKED_ATTRITION: Self = Self::from_static(
        "0bc630bf-2ee3-4eae-b272-b68d4ab80be7",
        "b6b22432-f3d9-468c-9359-b82a72791030",
    );
    pub const RANKED_CTF_3_CAPTURES: Self = Self::from_static(
        "4cb279b7-a064-4df6-9058-02cdc6825d93",
        "1392d27e-e7e3-41d9-93f9-420c66cff577",
    );
    pub const RANKED_CTF_5_CAPTURES: Self = Self::from_static(
        "507191c6-a492-4331-b2ae-a172101eb23e",
        "58052d54-b2d6-4006-baba-243a9d58c13d",
    );
    pub const RANKED_DOUBLES_ODDBALL: Self = Self::from_static(
        "9beb95dc-9fa2-4c6e-889f-d717f2adfe49",
        "75c45183-df50-405c-8fbc-bccc0f7eb375",
    );
    pub const RANKED_DOUBLES_SLAYER: Self = Self::from_static(
        "b0c65df9-0b2c-4040-b018-ad3e1baab832",
        "9e8f9dae-007d-4eb4-a131-4e5d526d9130",
    );
    pub const RANKED_KING_OF_THE_HILL: Self = Self::from_static(
        "88c22b1f-2d64-48b9-bab1-26fe4721fb23",
        "43e75f3a-eee5-4147-b9d3-19782fac58f8",
    );
    pub const RANKED_ODDBALL: Self = Self::from_static(
        "751bcc9d-aace-45a1-8d71-358f0bc89f7e",
        "227d4ffc-d67f-449a-8315-a1f82854d2ed",
    );
    pub const RANKED_ONE_FLAG_CTF: Self = Self::from_static(
        "18ac247d-7f86-4a59-9b47-9e74a6384ac2",
        "6dbe8411-cc9b-44ca-b680-32847677536a",
    );
    pub const RANKED_SLAYER: Self = Self::from_static(
        "c2d20d44-8606-4669-b894-afae15b3524f",
        "0091d411-f90d-44a7-aac3-ccc7ff2b131f",
    );
    pub const RANKED_STRONGHOLDS: Self = Self::from_static(
        "22b8a0eb-0d02-4eb3-8f56-5f63fc254f83",
        "7a6d2582-284c-4728-bec9-118e32cd0cc0",
    );
    pub const SLAYER_BTB: Self = Self::from_static(
        "920d628c-9eae-47a6-b96c-d141cf36ade2",
        "2a228d8e-cb58-4804-8e4c-926aae27c61d",
    );
    pub const SLAYER_BTB_FIESTA: Self = Self::from_static(
        "ab3c7acc-2af8-4a5e-b510-ddea47054c4a",
        "be364419-a424-48d0-a943-812ed6e184cc",
    );
    pub const SLAYER_BTB_HEAVIES: Self = Self::from_static(
        "64edc877-e2aa-4b83-b507-5d64ee4fefe9",
        "1627eba7-1d88-4d53-8b98-860812da5b0d",
    );
    pub const SLAYER_DOUBLES: Self = Self::from_static(
        "10a89f4f-9d37-4833-8db2-b95931f5eecd",
        "ba5aa4d4-f6c0-433e-b094-ae5f2703e241",
    );
    pub const SQUAD_CTF: Self = Self::from_static(
        "16188841-9cb2-4cf3-bb59-5139f9a737ab",
        "871ccd7c-8bf3-4513-a722-24f972f34d22",
    );
    pub const SQUAD_KING_OF_THE_HILL: Self = Self::from_static(
        "3899e110-91cd-4479-a8ad-5f8f8b91248d",
        "8f137484-6970-46c9-a4a9-c26ff0c232d4",
    );
    pub const SQUAD_ONE_FLAG_CTF: Self = Self::from_static(
        "4640310c-8a5c-4afa-bc39-98835e49d9f2",
        "046685be-2189-4ba2-8874-b5c7f2ee1615",
    );
    pub const SQUAD_SLAYER: Self = Self::from_static(
        "d73d459a-d63d-4a21-97f0-b1b156101d3c",
        "a2402448-53d3-42d0-8c53-393bf5ac055b",
    );
    pub const TOTAL_CONTROL_BTB: Self = Self::from_static(
        "34bac2c7-b6d7-4202-b634-1d770e5247a4",
        "341262d6-39c9-4aed-832d-86dbc94e0eca",
    );
    pub const TOTAL_CONTROL_BTB_FIESTA: Self = Self::from_static(
        "ac993227-14be-4aea-a055-8782665c4251",
        "cb71fdfe-faa4-4c06-8a43-d8aa023fcc22",
    );

    /// Creates a game-mode ID from an asset GUID and immutable version GUID.
    pub fn new(asset_id: impl Into<String>, version_id: impl Into<String>) -> Self {
        Self {
            asset_id: Cow::Owned(asset_id.into()),
            version_id: Cow::Owned(version_id.into()),
        }
    }

    const fn from_static(asset_id: &'static str, version_id: &'static str) -> Self {
        Self {
            asset_id: Cow::Borrowed(asset_id),
            version_id: Cow::Borrowed(version_id),
        }
    }

    pub fn asset_id(&self) -> &str {
        &self.asset_id
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
    }
}

/// A Halo Infinite map asset ID paired with an immutable version ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapId {
    asset_id: Cow<'static, str>,
    version_id: Cow<'static, str>,
}

impl MapId {
    pub const APOSTLE: Self = Self::from_static(
        "023507d1-7566-458d-b3d7-31627c40c01d",
        "ac0cd6db-2ce4-43df-91ef-7e5a90550a08",
    );
    pub const AQUARIUS: Self = Self::from_static(
        "c395f3ac-4614-45f9-a83a-56f69e8ae962",
        "dda9eaa7-441d-45ef-983b-bf9e89298be5",
    );
    pub const AQUARIUS_RANKED: Self = Self::from_static(
        "667072ae-ba00-4414-adda-8203e8c49295",
        "9cce4f7e-cd2e-470f-93eb-a2725958f8bd",
    );
    pub const ARGYLE: Self = Self::from_static(
        "2e35d8c0-8292-465c-8d69-35b5ff8b70db",
        "c4dffc26-537d-4162-a9d1-703798784c49",
    );
    pub const ARGYLE_RANKED: Self = Self::from_static(
        "78677080-db7a-429e-84fe-f041d8342c37",
        "042ca666-15c8-4b68-b7bc-7a1d2fbdf113",
    );
    pub const ARRIVAL: Self = Self::from_static(
        "beaf8ac9-fa85-4562-8def-28d9428dbddb",
        "e92c6ffa-741c-4c82-a253-657acd92c694",
    );
    pub const BAZAAR: Self = Self::from_static(
        "3e1e4cec-4f2c-44c6-b8d2-96b85c66c702",
        "f463f61f-719f-4e1d-9a55-8492cce69b0b",
    );
    pub const BEHEMOTH: Self = Self::from_static(
        "c3eabce0-2b2b-413c-96de-365a78846993",
        "95ea4511-8d7e-48d0-b18d-5c7fde89ed94",
    );
    pub const BREAKER: Self = Self::from_static(
        "6a147baf-b9c0-489e-8f04-8c6676c948d2",
        "735d5165-dc3c-46c5-9dfc-30ee9f6b8bb7",
    );
    pub const BREAKER_HEAVIES: Self = Self::from_static(
        "953dc3e2-943f-4ae3-abdd-b4be2a6a7a3a",
        "4c7e77db-39f3-4219-8389-a0b3eaaecdeb",
    );
    pub const BREAKER_ONE_FLAG_CTF: Self = Self::from_static(
        "66508493-003c-4f3d-9a2b-91f37eabcfa6",
        "70269654-ecb5-44de-a3bc-6b8b21e65787",
    );
    pub const BREAKPOINT: Self = Self::from_static(
        "fe7d8044-bc7f-4a05-a266-061382dfd8fa",
        "2599dda4-1ad7-4ecf-afe8-c601dbe789b2",
    );
    pub const CATALYST: Self = Self::from_static(
        "f7e8cde9-0c0a-487c-94a3-61bfa0f20465",
        "55753d9f-3cab-4421-8280-c3a877319d68",
    );
    pub const CATALYST_RANKED: Self = Self::from_static(
        "74b8c681-a3b0-422b-9fea-c111d3c979da",
        "523d7c04-367b-4160-8f03-59792c18a184",
    );
    pub const CHASM: Self = Self::from_static(
        "a455572d-3141-48bc-ac55-dac78d9b52c9",
        "d03834f3-acad-4217-8ecb-244839366f2e",
    );
    pub const CLIFFHANGER: Self = Self::from_static(
        "81274d6f-6a94-425a-a16e-3bdb1e2eea9d",
        "2cbfa179-2bd2-499d-a5bd-74bf2d14d05b",
    );
    pub const CLIFFSIDE: Self = Self::from_static(
        "4bffd021-92c0-422b-8b6e-8f595511458c",
        "9e4296fd-b785-4bea-a880-8a5f8b268d56",
    );
    pub const COMMAND: Self = Self::from_static(
        "2c9f3490-6be2-4d90-9015-02095651e91e",
        "a9f63a1c-3f6a-4125-a2a1-f4bbbb864e97",
    );
    pub const COMMAND_ASSAULT: Self = Self::from_static(
        "66b15c7c-d439-41b4-a965-575bd5c86c04",
        "93e2b110-16a4-44cc-a7b3-06991f285a4e",
    );
    pub const COMMAND_SENTRY_DEFENSE: Self = Self::from_static(
        "215eb485-d3b9-43d3-9390-42a9cc2b7db9",
        "304c033f-3b0e-4d03-8c03-927da02edeb2",
    );
    pub const CRASHOUT: Self = Self::from_static(
        "178ee422-43c1-4df8-be86-84f8bb0cc4d2",
        "c72d84c8-9077-48a2-a78c-38d6df82a5c2",
    );
    pub const CREDENCE: Self = Self::from_static(
        "0cc728d2-9b4d-4b80-95c9-18c77c095575",
        "a73940e7-210d-4aba-b039-ec51c006d146",
    );
    pub const CRITICAL_DEWPOINT: Self = Self::from_static(
        "bae4df14-4f4a-424c-aac1-2f795c807146",
        "35fdc4d3-7864-4600-adc9-847de372166a",
    );
    pub const CURFEW: Self = Self::from_static(
        "63d634be-0319-489d-8c21-9c4e012f664f",
        "397afd65-341c-46f0-9a32-caa46d995f0a",
    );
    pub const DAIMYO: Self = Self::from_static(
        "0590f221-1e5b-45ca-919c-df1b113eab3f",
        "60769158-fbd9-41be-84af-6d43be9b4363",
    );
    pub const DAWNBREAKER: Self = Self::from_static(
        "89dd4003-455c-4a1c-bcea-43acd514b20d",
        "93d0fd61-f115-449f-af8a-6de2b340b764",
    );
    pub const DEAD_WATER: Self = Self::from_static(
        "321271c8-529a-4a97-b303-093d839a6068",
        "4596b8d4-d97c-4ee9-a5f4-1b06464d56ce",
    );
    pub const DEADLOCK: Self = Self::from_static(
        "6f82df05-3b24-4746-8579-e0c5ef9e9d64",
        "f1f191c2-d388-4cf4-a177-b28f8229d80b",
    );
    pub const DEADLOCK_HEAVIES: Self = Self::from_static(
        "4fd75c74-5098-47ce-b6d3-4540df3f9b8e",
        "12d90f79-fe4e-4f21-adc8-a7bbd8d71dd6",
    );
    pub const DEADLOCK_SENTRY_DEFENSE: Self = Self::from_static(
        "34b81859-5cbc-4276-965e-8da33ebc824f",
        "f8741b7d-c01a-43df-a008-f4139259733b",
    );
    pub const DOMICILE: Self = Self::from_static(
        "921aebb1-783d-45e4-bacd-7ad869fa8dae",
        "6a2f9336-a5cd-41c5-ab0d-0da39ef351c1",
    );
    pub const DREDGE: Self = Self::from_static(
        "e4bb06db-065f-4902-b93b-d8dac315eac4",
        "55402a49-0dba-495a-9a21-7346a1b79e81",
    );
    pub const ELEVATION: Self = Self::from_static(
        "76043dc6-2724-45e2-9b5a-6fe2e75da588",
        "c131c7eb-c2da-407d-b1d4-b4eb9e65037d",
    );
    pub const EMPYREAN: Self = Self::from_static(
        "d035fc3e-f298-4c14-9487-465be2e1dc1f",
        "8d5eb886-a41d-4093-827e-2cbdc75c651e",
    );
    pub const EMPYREAN_RANKED: Self = Self::from_static(
        "70dd38c5-2eb7-4db3-8901-0dfca292ff18",
        "8c4ab697-8630-47bf-b8a8-7d49a5e91b3c",
    );
    pub const EXHUMED: Self = Self::from_static(
        "354b1633-1b47-4e0f-9a43-15ebf0acef0c",
        "65679de1-145f-494b-89b2-8338e6ee019f",
    );
    pub const EXILED: Self = Self::from_static(
        "9bb8d9df-ff6c-4a6e-b151-436bbb2c0907",
        "c9bb9a73-c790-436d-8f95-5409ac5a183a",
    );
    pub const FIRST_LIGHT: Self = Self::from_static(
        "c85ccdc7-8368-482f-8ca5-8b8d5d9a096d",
        "cc1cefc8-d73f-478a-b7c9-0227e1ebee91",
    );
    pub const FLOOD_GULCH: Self = Self::from_static(
        "7097bc4f-efcf-4c5a-a96e-4ddb03e84d2a",
        "ea3b126b-a2cf-4cdd-9d02-de154417b286",
    );
    pub const FORBIDDEN: Self = Self::from_static(
        "87c03bfd-2db3-4a5b-bbf9-d5369c5894d1",
        "500d0305-72eb-47a4-aba5-f599e221cf00",
    );
    pub const FOREST: Self = Self::from_static(
        "e8d56863-9ad4-4efe-9059-81270884589c",
        "740b1651-e6cd-4172-a2a9-1c1710fa022f",
    );
    pub const FORGE_SPACE: Self = Self::from_static(
        "76669255-697d-48c9-a802-7ff2ec8257f1",
        "b8abf687-4e95-4846-83c7-33e779eed33e",
    );
    pub const FORTITUDE: Self = Self::from_static(
        "1ede38fa-4d30-4dfa-a8b7-5d08bf4e46e3",
        "ad5eb2a0-1909-425e-a7de-5faa34a9d1e7",
    );
    pub const FORTITUDE_ASSAULT: Self = Self::from_static(
        "ecce7405-53c2-4fb1-a57f-33af084f37d7",
        "8cae5866-e28f-48b4-ac13-e7312212f80c",
    );
    pub const FORTITUDE_HEAVIES: Self = Self::from_static(
        "305b1bdd-9a7b-4975-bacf-8bd63c8c13d2",
        "615d7c10-8473-45ca-9c87-88c5d4f7acff",
    );
    pub const FORTRESS_RANKED: Self = Self::from_static(
        "a54808fb-9bf5-432a-a3c3-f76cbea944c1",
        "f8fe5de8-694e-4787-9ece-dea86b37e6be",
    );
    pub const FRAGMENTATION: Self = Self::from_static(
        "068f41f4-e3dd-4bec-b297-d2ded85ab54b",
        "2f0b4c72-51e5-4115-b302-0bae4b4df7dd",
    );
    pub const FRAGMENTATION_HEAVIES: Self = Self::from_static(
        "0d849a52-fedb-4aea-b5a3-caee268f1f49",
        "f0dcbaf5-b10f-4f13-bb76-2e789ed4c18a",
    );
    pub const FRAGMENTATION_SENTRY_DEFENSE: Self = Self::from_static(
        "10d59d28-e00a-4bf1-9890-3a3e6cbbc64c",
        "8cc62b67-a062-490b-8b74-af91ebff4607",
    );
    pub const GONDOLAS: Self = Self::from_static(
        "c4cd9e46-3666-4d89-98ec-3b7b2c7005fb",
        "59b54acf-d016-41e6-a941-732416a39f25",
    );
    pub const GOTHIC: Self = Self::from_static(
        "0d5cb522-ab18-4348-8a04-36948ca1f2e1",
        "26b1ccd4-504f-4c81-99bb-66b3d7d72192",
    );
    pub const GYRE: Self = Self::from_static(
        "2aba3426-083c-42a9-b469-02898d4d0c62",
        "81957761-5b8b-4053-a84a-2203486133cf",
    );
    pub const HARVEST: Self = Self::from_static(
        "ff6b8a12-6b95-4eac-8ee1-9ba4c985c2e0",
        "e7d01a4c-f5dc-4286-85d8-de37ab2d8841",
    );
    pub const HARVESTERS: Self = Self::from_static(
        "8168a385-24f7-4d2b-8b1d-bfa3c741401f",
        "9042df75-fcda-497b-9d95-93ad33b5188a",
    );
    pub const HIGHPOWER: Self = Self::from_static(
        "33c6505d-1cfa-43c6-9fa9-311eb0502ad9",
        "d550b178-a436-4821-ba21-f11a57f35b63",
    );
    pub const HIGHPOWER_HEAVIES: Self = Self::from_static(
        "ecbb3aa1-6227-43c7-8cf8-ce62d1a988f0",
        "8b13a317-c677-4f72-b5de-ce6c816ef20b",
    );
    pub const HIGHPOWER_SENTRY_DEFENSE: Self = Self::from_static(
        "142a5e23-46de-4429-a232-aac4e6459a11",
        "9fc3a03e-b97b-4c7c-b6aa-e7d8608c10c7",
    );
    pub const HOUSE_OF_RECKONING: Self = Self::from_static(
        "eaf608f0-6ec3-444f-a51a-9c1de5d0bf5c",
        "681d9ead-df2c-45d6-a828-a7d9e2e582cd",
    );
    pub const ILLUSION: Self = Self::from_static(
        "9e821f5e-042f-407c-97f3-de165b1cdb26",
        "b1f66098-0095-472e-947f-de171998fc10",
    );
    pub const IMMOLATE: Self = Self::from_static(
        "47823612-9de0-4ca9-8a95-b3a6ebd7ca91",
        "ef53d2cc-fd2f-45a2-9a0a-a77082e4911f",
    );
    pub const INSOLENCE: Self = Self::from_static(
        "d5c5eb4f-0dcb-4677-a866-eae0dcbfde9b",
        "b574a3ab-0329-4e00-8e84-0db796c2d5df",
    );
    pub const INSOLENCE_ASSAULT: Self = Self::from_static(
        "32fcf611-9e8f-4475-894c-acd65fbf39b1",
        "d7db089e-03b3-4a84-9c45-5965017a7fb9",
    );
    pub const INSOLENCE_HEAVIES: Self = Self::from_static(
        "2a339c65-5128-4457-88d4-0906e265034e",
        "32583c91-936f-4ecc-931c-a7df4626783f",
    );
    pub const INTERFERENCE: Self = Self::from_static(
        "654dff62-d618-496a-8914-06ab73d991e3",
        "87e7bd29-9914-48f3-81e0-38ad200e1e4e",
    );
    pub const JAROK_BRIDGE: Self = Self::from_static(
        "7fbce06f-0d2b-498a-bf7d-f86e6e224820",
        "f1eb96a2-27d7-43b8-b4d4-1f43a3b21c45",
    );
    pub const KUSINI_BAY: Self = Self::from_static(
        "89f3b8ad-6bbf-4652-8d67-8e5330294de4",
        "ee7540aa-e504-4372-8e04-3e583a3359aa",
    );
    pub const LAST_BROADCAST: Self = Self::from_static(
        "67c349f5-d7cc-49a0-9cf0-6afba73b18be",
        "f2f13ee6-cf73-4246-b011-e432f2997de4",
    );
    pub const LAST_ROAD: Self = Self::from_static(
        "24d6a48b-d78b-49c7-8b24-338fa3508a32",
        "b73af8d8-37e4-485a-ba46-052867138300",
    );
    pub const LATTICE_RANKED: Self = Self::from_static(
        "1a6cfc2e-ec86-48e1-9464-1ce1bff6ed48",
        "3a382104-9b89-4e6e-aa18-0affaa98f478",
    );
    pub const LAUNCH_SITE: Self = Self::from_static(
        "56a11b8c-64d1-4537-8893-a9241e4d5b93",
        "1cd21d5a-a57e-4d42-997d-ff95ca0e32fc",
    );
    pub const LIVE_FIRE: Self = Self::from_static(
        "6c01f693-c968-4a71-b157-efc35ffcf71f",
        "ce0d50f3-e756-4aa0-a81c-85e47c83aa8b",
    );
    pub const LIVE_FIRE_RANKED: Self = Self::from_static(
        "309253f8-7a75-48ff-83e1-e7fb3db2ac47",
        "86a644f0-5063-40b8-b601-ce361439da72",
    );
    pub const LONGSHORE: Self = Self::from_static(
        "a6fb1ff7-3130-454c-b484-270c4ce07bf3",
        "b6217ea0-cf2e-47e7-8e5a-823ea2d05441",
    );
    pub const NADAIR: Self = Self::from_static(
        "6dbd1c0d-a6c2-4697-8453-f0799d941741",
        "b7fe7b46-9a4a-44b7-9dde-17515aeb4d7a",
    );
    pub const OASIS: Self = Self::from_static(
        "7aa6fed5-4c21-43dd-b740-2fae8c971517",
        "97da30e7-14d5-48db-b15b-91b453286ffd",
    );
    pub const OASIS_HEAVIES: Self = Self::from_static(
        "7f56a242-c93a-4a19-b084-a9bf9cdd0246",
        "5dcd3924-2e17-4369-81e1-ca0801ed89bb",
    );
    pub const OASIS_SENTRY_DEFENSE: Self = Self::from_static(
        "052956b4-06d1-4f78-9938-6b43c66bb223",
        "806de5c2-1a74-44dc-9179-50bb390df384",
    );
    pub const OBITUARY: Self = Self::from_static(
        "a289bafe-102e-4363-98f7-80b596007338",
        "93845a4c-0ace-4161-86cb-9d0c190d07da",
    );
    pub const OBITUARY_ASSAULT: Self = Self::from_static(
        "f2daaa7f-5bd0-46fc-b00c-feb970675125",
        "586e6852-0e69-40cd-8a85-1364b6111ab5",
    );
    pub const OBITUARY_HEAVIES: Self = Self::from_static(
        "e3681516-2930-491c-b94f-7dbfa161e000",
        "4ee8fc3e-cdca-44d7-8e59-d677d7afaa60",
    );
    pub const OPULENCE: Self = Self::from_static(
        "255bbe78-b191-476e-b0ae-0763c3bc2f44",
        "b29f6e87-b4ce-4423-8277-e749bedbb813",
    );
    pub const ORIGIN_RANKED: Self = Self::from_static(
        "46a8319c-2c63-46ee-9382-788906dcb049",
        "82e20c0a-ca3d-450a-a797-f5ed277a7dc2",
    );
    pub const PERDITION: Self = Self::from_static(
        "be8131fc-8839-4448-b65c-1fb46dd077ef",
        "a63fe01d-a1c2-4955-8ec3-d8666fb31496",
    );
    pub const PERILOUS: Self = Self::from_static(
        "c5ac9f12-660e-4f1a-83e7-2e7536bbcb04",
        "7c5df4a3-5c39-4984-a483-84f2c78190f3",
    );
    pub const PRISM: Self = Self::from_static(
        "2fdb8370-e5ac-4a1a-bdce-a08bc738b9ad",
        "867d213f-2e3a-436b-8dfd-6f0347decbb7",
    );
    pub const PROMENADE: Self = Self::from_static(
        "6edb3f62-aed7-49a5-b4c0-b44e9d010854",
        "4e1f18d0-e085-46f9-9cd2-c302e74ddb50",
    );
    pub const RAT_S_NEST: Self = Self::from_static(
        "133c0185-24ed-4bc2-b834-62db5c936257",
        "cfc6d71c-306b-4088-abc0-cadd8754a8a1",
    );
    pub const RECHARGE: Self = Self::from_static(
        "2b6d2baf-7645-4e16-8a80-c7006f595812",
        "03134a7d-1dd1-499a-a7e1-3d1319eca633",
    );
    pub const RECHARGE_RANKED: Self = Self::from_static(
        "336b5174-3579-4fd8-b2f0-922e4a5f7628",
        "c0c6705e-167b-4335-afe0-2bafc7290f40",
    );
    pub const RECOVERY: Self = Self::from_static(
        "1bc0e2a7-9d6d-4771-9574-9978e7c9292c",
        "bc4a3d27-ec96-49af-8ae0-98015f548397",
    );
    pub const REFUGE: Self = Self::from_static(
        "41217472-3020-4bd8-bce9-b2a2b0d50896",
        "99c96da2-961a-4436-8d29-9e2a17a16c7f",
    );
    pub const REFUGE_HEAVIES: Self = Self::from_static(
        "8aa45646-d527-47cc-affe-deac726a4af5",
        "4fde7a23-b792-4d81-a495-7c0b03c47834",
    );
    pub const RENDEZVOUS: Self = Self::from_static(
        "a778ae21-a8ae-4569-acb5-898efbd4b3f3",
        "4ebe71cd-9001-4fcf-bb71-d29bded96b7b",
    );
    pub const SALVATION: Self = Self::from_static(
        "cd08bc7a-7ba5-4502-be87-c58b641fc94d",
        "ef803442-9dbe-4ac3-b848-8eba21d4845b",
    );
    pub const SCARR: Self = Self::from_static(
        "ccb81e1f-ce22-4017-97bb-f46b181eb8f7",
        "e6e10a2f-c35f-49ed-954b-24e6c5fdfb38",
    );
    pub const SCARR_HEAVIES: Self = Self::from_static(
        "c5d5e3f4-6021-4590-aacc-a78333be6ea0",
        "4bd03ddc-597b-43aa-9449-9484c4b6cde4",
    );
    pub const SCARR_SENTRY_DEFENSE: Self = Self::from_static(
        "64511055-6442-4797-8ec4-028c33996fe6",
        "7eab053a-bcf2-4afa-8cad-9ceed63f4d2f",
    );
    pub const SERENITY_RANKED: Self = Self::from_static(
        "1de0bf60-e446-4fb9-970f-d0e54fc6c74a",
        "10f3c5bd-8eb7-4bad-8360-a66a581715ce",
    );
    pub const SHIRO: Self = Self::from_static(
        "2890782c-0a33-4f2c-a468-e3a7d6cd6db4",
        "eeec3be0-1bdf-4703-96dd-2633280c96c8",
    );
    pub const SNOWBOUND: Self = Self::from_static(
        "410f1c01-aca6-4567-9df5-9b16bd550cb2",
        "e7aa799d-7007-4ab8-a5b6-08926f768c2c",
    );
    pub const SOLITUDE: Self = Self::from_static(
        "f1cc3b4e-471c-4ec5-b855-1db7d9e6ce42",
        "cc6dfb16-7782-4995-a74a-98e86051fcdf",
    );
    pub const SOLITUDE_RANKED: Self = Self::from_static(
        "4a5e5612-2b2e-4375-a0b3-9335a68815f3",
        "77710e1b-d9bc-42fd-a74b-918c27387783",
    );
    pub const STARBOARD: Self = Self::from_static(
        "7a9265af-a880-487b-8829-68d88fcfb145",
        "6147aa00-84e8-44ac-93c0-8638506349f1",
    );
    pub const STREETS: Self = Self::from_static(
        "9c7b0b0f-e933-4c2d-9d4a-3e4500d0de99",
        "f39c10d7-e191-42be-b9c5-1502ccd2eb7a",
    );
    pub const STREETS_RANKED: Self = Self::from_static(
        "e23ea388-9bcb-4180-a0dc-fbe987751b9e",
        "bc130bc8-6610-458d-b04a-ead6392824c4",
    );
    pub const SUNKEN: Self = Self::from_static(
        "b66992eb-0bc9-4ec5-8e43-f850cb7317f3",
        "469500ac-d9e9-44de-9cff-b3b2758e5c1e",
    );
    pub const SYLVANUS: Self = Self::from_static(
        "95b69e4b-485f-4c6c-9b00-4bd68c94c1e9",
        "d6a434d9-0860-451f-a971-e9cfead782ae",
    );
    pub const THRESHOLD: Self = Self::from_static(
        "ddbb3a00-b109-4703-af07-00433512af38",
        "1e198b75-0371-4904-b44e-59af97c17d7d",
    );
    pub const THUNDERHEAD: Self = Self::from_static(
        "28a3ac28-f69d-4fa9-9ebf-a0449c89c8da",
        "5f93b32d-cdae-4742-b4ad-09bc3e0720f0",
    );
    pub const THUNDERHEAD_ASSAULT: Self = Self::from_static(
        "97361eb7-cf18-468a-8152-5b8fafcb27e4",
        "2264b3a7-5969-43f3-892d-225299d48159",
    );
    pub const THUNDERHEAD_HEAVIES: Self = Self::from_static(
        "37bc3df6-93e8-4d74-b16e-5ceaa30ebc23",
        "7deeaaa6-5487-4bc4-bc6e-c9e94041968f",
    );
    pub const TIMBERLAND_EVOLVED: Self = Self::from_static(
        "1231d3f0-2363-4d28-8047-717a069fb0e4",
        "c6fd3ecd-e503-42b9-9437-389e72db33d8",
    );
    pub const VACANCY_RANKED: Self = Self::from_static(
        "6a1e8432-88ae-4430-8f7d-9ffefc97cc8d",
        "97a448f9-0734-426b-ab10-b67fbd75f85f",
    );
    pub const VALLAHEIM: Self = Self::from_static(
        "688c2033-35de-461e-9394-a32c665c964c",
        "353a79a0-831d-4c27-9496-799d111b5b89",
    );
    pub const WATERWORKS: Self = Self::from_static(
        "0661af5e-8b6d-44c0-bb7c-9c76cdcc624d",
        "1b61d905-687d-4041-86a1-beb5259c6cff",
    );
    pub const WATERWORKS_ASSAULT: Self = Self::from_static(
        "68df8895-7f8c-4668-8624-92f35f9559f6",
        "07fc5222-2af1-4373-933a-2ed679f21501",
    );
    pub const WAVELENGTH: Self = Self::from_static(
        "72339fd1-c61a-4cbb-a876-0ffdb877a899",
        "cd8bf101-9e9c-40d7-a048-e6647e94a12f",
    );
    pub const YOSAI: Self = Self::from_static(
        "b902c6a0-6140-40bd-afc3-74b9e0a5916c",
        "fbd54b72-4d1e-4605-b8ee-1c7c9b2e982b",
    );

    /// Creates a map ID from an asset GUID and immutable version GUID.
    pub fn new(asset_id: impl Into<String>, version_id: impl Into<String>) -> Self {
        Self {
            asset_id: Cow::Owned(asset_id.into()),
            version_id: Cow::Owned(version_id.into()),
        }
    }

    const fn from_static(asset_id: &'static str, version_id: &'static str) -> Self {
        Self {
            asset_id: Cow::Borrowed(asset_id),
            version_id: Cow::Borrowed(version_id),
        }
    }

    pub fn asset_id(&self) -> &str {
        &self.asset_id
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UgcAssetKind {
    Map,
    Playlist,
    GameMode,
    MapModePair,
    Film,
    Prefab,
    EngineGameVariant,
}

impl UgcAssetKind {
    /// The `assetKind` filter value [`HaloInfiniteClient::search_assets`](crate::clients::hi::HaloInfiniteClient::search_assets) expects.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Map => "Map",
            Self::Playlist => "Playlist",
            Self::GameMode => "UgcGameVariant",
            Self::MapModePair => "MapModePair",
            Self::Film => "Film",
            Self::Prefab => "Prefab",
            Self::EngineGameVariant => "EngineGameVariant",
        }
    }

    /// The `/hi/{kind}/{asset_id}` URL path segment Halo expects — distinct casing from
    /// [`Self::as_str`], which is the `assetKind` filter value used by `search_assets`.
    pub(crate) const fn path_segment(self) -> &'static str {
        match self {
            Self::Map => "maps",
            Self::Playlist => "playlists",
            Self::GameMode => "ugcGameVariants",
            Self::MapModePair => "mapModePairs",
            Self::Film => "films",
            Self::Prefab => "prefabs",
            Self::EngineGameVariant => "engineGameVariants",
        }
    }
}

/// A UGC asset envelope with no kind-specific `CustomData` typed yet.
///
/// Halo's asset envelope (asset id, version id, name, description, files) is uniform across
/// kinds; only `CustomData` varies. Kinds without a bespoke wrapper (see
/// [`HaloInfiniteClient::map`](crate::clients::hi::HaloInfiniteClient::map)/
/// [`HaloInfiniteClient::mode`](crate::clients::hi::HaloInfiniteClient::mode) for kinds that do)
/// deserialize via this type, which ignores whatever `CustomData` the response carries.
#[derive(Debug, Clone, Deserialize)]
pub struct UgcAsset {
    #[serde(flatten)]
    pub asset: AssetLink,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UgcSearchResults {
    #[serde(rename = "EstimatedTotal")]
    pub estimated_total: u32,
    #[serde(rename = "Start")]
    pub start: u32,
    #[serde(rename = "ResultCount")]
    pub result_count: u32,
    #[serde(rename = "Results")]
    pub results: Vec<UgcSearchResult>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct UgcSearchResult {
    #[serde(rename = "AssetId")]
    pub asset_id: String,
    #[serde(rename = "AssetVersionId")]
    pub version_id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "AssetKind")]
    pub asset_kind: i32,
    #[serde(rename = "Tags", deserialize_with = "deserialize_null_default")]
    pub tags: Vec<String>,
    #[serde(rename = "ThumbnailUrl")]
    pub thumbnail_url: String,
    #[serde(rename = "OriginalAuthor")]
    pub original_author: String,
    #[serde(rename = "Contributors", deserialize_with = "deserialize_null_default")]
    pub contributors: Vec<String>,
    #[serde(rename = "Likes")]
    pub likes: i64,
    #[serde(rename = "Bookmarks")]
    pub bookmarks: i64,
    #[serde(rename = "PlaysRecent")]
    pub plays_recent: i64,
    #[serde(rename = "PlaysAllTime")]
    pub plays_all_time: i64,
    #[serde(rename = "NumberOfObjects")]
    pub number_of_objects: i64,
    #[serde(rename = "AverageRating")]
    pub average_rating: f64,
    #[serde(rename = "NumberOfRatings")]
    pub number_of_ratings: i64,
    /// The catalog that owns the asset. Halo-owned assets use home `2`.
    #[serde(rename = "AssetHome")]
    pub asset_home: Option<i32>,
}

/// Response body from the playlist CSR endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CsrRecords {
    #[serde(rename = "Value")]
    pub records: Vec<CsrRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsrRecord {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "ResultCode")]
    pub result_code: i32,
    #[serde(rename = "Result")]
    pub result: CsrRecordResult,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsrRecordResult {
    #[serde(rename = "Current")]
    pub current: CsrRecordRanking,
    #[serde(default, rename = "SeasonMax")]
    pub season_max: CsrRecordRanking,
    #[serde(rename = "AllTimeMax")]
    pub peak: CsrRecordRanking,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CsrRecordRanking {
    /// Numeric CSR value, or `-1` if the player is unranked in this playlist.
    #[serde(rename = "Value")]
    pub value: i32,
    #[serde(rename = "MeasurementMatchesRemaining")]
    pub measurement_matches_remaining: i32,
    #[serde(rename = "Tier")]
    pub tier: String,
    #[serde(rename = "TierStart")]
    pub tier_start: i32,
    /// 0-indexed sub-tier within `tier`. Not meaningful for Onyx, which reports `value`.
    #[serde(rename = "SubTier")]
    pub sub_tier: i32,
    #[serde(rename = "NextTier")]
    pub next_tier: String,
    #[serde(rename = "NextTierStart")]
    pub next_tier_start: i32,
    #[serde(rename = "NextSubTier")]
    pub next_sub_tier: i32,
    #[serde(rename = "InitialMeasurementMatches")]
    pub initial_measurement_matches: i32,
    #[serde(default, rename = "DemotionProtectionMatchesRemaining")]
    pub demotion_protection_matches_remaining: i32,
    #[serde(default, rename = "InitialDemotionProtectionMatches")]
    pub initial_demotion_protection_matches: i32,
}

const RANK_ICON_BASE_URL: &str = "https://trackercdn.com/cdn/tracker.gg/halo-infinite/skills/c";

impl CsrRecordRanking {
    pub fn is_unranked(&self) -> bool {
        self.value == -1
    }

    /// Returns a rank icon URL from tracker.gg's public CDN (not a Halo Waypoint endpoint, and
    /// not officially documented — no authentication required, but subject to change without
    /// notice).
    ///
    /// Icons follow the pattern `{tier}-{subtier}.png` (lowercase tier, 1-indexed subtier),
    /// except Onyx, which has no subtiers and uses a bare `onyx.png`.
    pub fn rank_icon_url(&self) -> String {
        if self.is_unranked() {
            return Self::unranked_icon_url();
        }
        let tier = self.tier.to_lowercase();
        if tier == "onyx" {
            format!("{RANK_ICON_BASE_URL}/onyx.png")
        } else {
            format!("{RANK_ICON_BASE_URL}/{tier}-{}.png", self.sub_tier + 1)
        }
    }

    /// Returns the "unranked" rank icon URL, for when there is no [`CsrRecordRanking`] at all
    /// (e.g. a playlist CSR lookup failed) rather than one whose [`Self::is_unranked`].
    pub fn unranked_icon_url() -> String {
        format!("{RANK_ICON_BASE_URL}/unranked.png")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchesPrivacy {
    #[serde(rename = "MatchmadeGames")]
    pub matchmade_games: i32,
    #[serde(rename = "OtherGames")]
    pub other_games: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacySetting {
    Show,
    Hide,
    Unknown(i32),
}

impl PrivacySetting {
    pub fn from_code(code: i32) -> Self {
        match code {
            1 => Self::Show,
            2 => Self::Hide,
            other => Self::Unknown(other),
        }
    }
}

impl MatchesPrivacy {
    pub fn matchmade_setting(&self) -> PrivacySetting {
        PrivacySetting::from_code(self.matchmade_games)
    }

    pub fn other_setting(&self) -> PrivacySetting {
        PrivacySetting::from_code(self.other_games)
    }
}

/// A page of a player's match history.
#[derive(Debug, Clone, Deserialize)]
pub struct PlayerMatchHistory {
    #[serde(rename = "Results")]
    pub results: Vec<MatchHistoryEntry>,
    #[serde(rename = "ResultCount")]
    pub result_count: i32,
}

/// A player's or team's result in a completed match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchOutcome {
    Tie,
    Win,
    Loss,
    DidNotFinish,
    Unknown(i32),
}

impl MatchOutcome {
    pub const fn from_code(code: i32) -> Self {
        match code {
            1 => Self::Tie,
            2 => Self::Win,
            3 => Self::Loss,
            4 => Self::DidNotFinish,
            other => Self::Unknown(other),
        }
    }

    pub const fn code(self) -> i32 {
        match self {
            Self::Tie => 1,
            Self::Win => 2,
            Self::Loss => 3,
            Self::DidNotFinish => 4,
            Self::Unknown(code) => code,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tie => "Tie",
            Self::Win => "Victory",
            Self::Loss => "Defeat",
            Self::DidNotFinish => "Did not finish",
            Self::Unknown(_) => "Unknown",
        }
    }
}

impl<'de> Deserialize<'de> for MatchOutcome {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from_code(i32::deserialize(deserializer)?))
    }
}

impl std::fmt::Display for MatchOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchHistoryEntry {
    #[serde(rename = "MatchId")]
    pub match_id: String,
    #[serde(rename = "LastTeamId")]
    pub last_team_id: i32,
    #[serde(rename = "Outcome")]
    pub outcome: MatchOutcome,
    #[serde(rename = "Rank")]
    pub rank: i32,
    #[serde(rename = "PresentAtEndOfMatch")]
    pub present_at_end: bool,
    #[serde(rename = "MatchInfo")]
    pub info: MatchInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchInfo {
    #[serde(rename = "StartTime")]
    pub start_time: DateTime<Utc>,
    #[serde(rename = "EndTime")]
    pub end_time: DateTime<Utc>,
    #[serde(rename = "Duration")]
    pub duration: String,
    /// Time the match was actually playable, as an ISO-8601 duration.
    #[serde(default, rename = "PlayableDuration")]
    pub playable_duration: Option<String>,
    #[serde(rename = "GameVariantCategory")]
    pub game_variant_category: i32,
    #[serde(default, rename = "LifecycleMode")]
    pub lifecycle_mode: i32,
    #[serde(default, rename = "GameplayInteraction")]
    pub gameplay_interaction: i32,
    #[serde(default, rename = "LevelId")]
    pub level_id: Option<String>,
    #[serde(default, rename = "SeasonId")]
    pub season_id: Option<String>,
    #[serde(rename = "MapVariant")]
    pub map_variant: Option<MatchAssetLink>,
    #[serde(rename = "UgcGameVariant")]
    pub ugc_game_variant: Option<MatchAssetLink>,
    #[serde(rename = "Playlist")]
    pub playlist: Option<MatchPlaylist>,
    #[serde(default, rename = "PlaylistExperience")]
    pub playlist_experience: Option<i32>,
    #[serde(default, rename = "TeamsEnabled")]
    pub teams_enabled: bool,
    #[serde(default, rename = "TeamScoringEnabled")]
    pub team_scoring_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchPlaylist {
    #[serde(rename = "AssetId")]
    pub asset_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchAssetLink {
    #[serde(rename = "AssetId")]
    pub asset_id: String,
    #[serde(rename = "VersionId")]
    pub version_id: String,
    #[serde(rename = "AssetKind")]
    pub asset_kind: i32,
}

/// Detailed scoreboard returned for one match.
#[derive(Debug, Clone, Deserialize)]
pub struct MatchStats {
    #[serde(rename = "MatchId")]
    pub match_id: String,
    #[serde(rename = "MatchInfo")]
    pub info: MatchInfo,
    #[serde(rename = "Players")]
    pub players: Vec<MatchPlayer>,
    #[serde(rename = "Teams")]
    pub teams: Vec<MatchTeam>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchPlayer {
    #[serde(rename = "PlayerId")]
    pub player_id: String,
    /// Player type code. `1` denotes a human; bots use other values (see [`Self::is_human`]).
    #[serde(rename = "PlayerType")]
    pub player_type: i32,
    #[serde(default, rename = "BotAttributes")]
    pub bot_attributes: Option<BotAttributes>,
    #[serde(rename = "LastTeamId")]
    pub last_team_id: i32,
    #[serde(rename = "Outcome")]
    pub outcome: MatchOutcome,
    #[serde(rename = "Rank")]
    pub rank: i32,
    #[serde(default, rename = "ParticipationInfo")]
    pub participation_info: Option<ParticipationInfo>,
    #[serde(rename = "PlayerTeamStats")]
    pub team_stats: Vec<PlayerTeamStats>,
}

impl MatchPlayer {
    /// Returns `true` when this is a human player rather than a bot.
    pub fn is_human(&self) -> bool {
        self.player_type == 1
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerTeamStats {
    #[serde(rename = "TeamId")]
    pub team_id: i32,
    #[serde(rename = "Stats")]
    pub stats: MatchStatsBlock,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchTeam {
    #[serde(rename = "TeamId")]
    pub team_id: i32,
    #[serde(rename = "Outcome")]
    pub outcome: MatchOutcome,
    #[serde(rename = "Rank")]
    pub rank: i32,
    #[serde(rename = "Stats")]
    pub stats: MatchStatsBlock,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MatchStatsBlock {
    #[serde(rename = "CoreStats")]
    pub core: MatchCoreStats,
    /// Typed mode-specific stat blocks; only the one matching the match's mode is populated.
    #[serde(flatten)]
    pub mode_stats: ModeStats,
}

/// Per-player or per-team core stats for a single match.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MatchCoreStats {
    #[serde(rename = "Score")]
    pub score: i64,
    #[serde(rename = "PersonalScore")]
    pub personal_score: i64,
    #[serde(rename = "RoundsWon")]
    pub rounds_won: i64,
    #[serde(rename = "RoundsLost")]
    pub rounds_lost: i64,
    #[serde(rename = "RoundsTied")]
    pub rounds_tied: i64,
    #[serde(rename = "Kills")]
    pub kills: i64,
    #[serde(rename = "Deaths")]
    pub deaths: i64,
    #[serde(rename = "Assists")]
    pub assists: i64,
    #[serde(rename = "KDA")]
    pub kda: f64,
    #[serde(rename = "Suicides")]
    pub suicides: i64,
    #[serde(rename = "Betrayals")]
    pub betrayals: i64,
    /// Average life duration, as an ISO-8601 duration string.
    #[serde(rename = "AverageLifeDuration")]
    pub average_life_duration: String,
    #[serde(rename = "GrenadeKills")]
    pub grenade_kills: i64,
    #[serde(rename = "HeadshotKills")]
    pub headshot_kills: i64,
    #[serde(rename = "MeleeKills")]
    pub melee_kills: i64,
    #[serde(rename = "PowerWeaponKills")]
    pub power_weapon_kills: i64,
    #[serde(rename = "ShotsFired")]
    pub shots_fired: i64,
    #[serde(rename = "ShotsHit")]
    pub shots_hit: i64,
    #[serde(rename = "Accuracy")]
    pub accuracy: f64,
    #[serde(rename = "DamageDealt")]
    pub damage_dealt: i64,
    #[serde(rename = "DamageTaken")]
    pub damage_taken: i64,
    #[serde(rename = "CalloutAssists")]
    pub callout_assists: i64,
    #[serde(rename = "VehicleDestroys")]
    pub vehicle_destroys: i64,
    #[serde(rename = "DriverAssists")]
    pub driver_assists: i64,
    #[serde(rename = "Hijacks")]
    pub hijacks: i64,
    #[serde(rename = "EmpAssists")]
    pub emp_assists: i64,
    #[serde(rename = "MaxKillingSpree")]
    pub max_killing_spree: i64,
    #[serde(rename = "Medals")]
    pub medals: Vec<StatAward>,
    #[serde(rename = "PersonalScores")]
    pub personal_scores: Vec<StatAward>,
    #[serde(rename = "Spawns")]
    pub spawns: i64,
    #[serde(rename = "ObjectivesCompleted")]
    pub objectives_completed: i64,
}

/// Theater-film metadata and downloadable chunk inventory for a match.
#[derive(Debug, Clone, Deserialize)]
pub struct FilmManifest {
    #[serde(rename = "FilmStatusBond")]
    pub status: i32,
    #[serde(rename = "CustomData")]
    pub custom_data: FilmCustomData,
    #[serde(rename = "BlobStoragePathPrefix")]
    pub blob_storage_path_prefix: String,
    #[serde(rename = "AssetId")]
    pub asset_id: String,
}

impl FilmManifest {
    pub fn chunk_url(&self, chunk: &FilmChunk) -> String {
        format!(
            "{}/{}",
            self.blob_storage_path_prefix.trim_end_matches('/'),
            chunk.file_relative_path.trim_start_matches('/')
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilmCustomData {
    #[serde(rename = "FilmLength")]
    pub film_length: i64,
    #[serde(rename = "Chunks")]
    pub chunks: Vec<FilmChunk>,
    #[serde(rename = "HasGameEnded")]
    pub has_game_ended: bool,
    #[serde(rename = "ManifestRefreshSeconds")]
    pub manifest_refresh_seconds: i64,
    #[serde(rename = "MatchId")]
    pub match_id: String,
    #[serde(rename = "FilmMajorVersion")]
    pub film_major_version: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilmChunk {
    #[serde(rename = "Index")]
    pub index: i32,
    #[serde(rename = "ChunkStartTimeOffsetMilliseconds")]
    pub start_time_offset_ms: i64,
    #[serde(rename = "DurationMilliseconds")]
    pub duration_ms: i64,
    #[serde(rename = "ChunkSize")]
    pub size: i64,
    #[serde(rename = "FileRelativePath")]
    pub file_relative_path: String,
    #[serde(rename = "ChunkType")]
    pub chunk_type: i32,
}

#[derive(Debug, Clone)]
pub struct FilmChunkData {
    pub metadata: FilmChunk,
    /// Decompressed film data.
    pub data: Vec<u8>,
}

/// Response body from the matchmade service record endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ServiceRecord {
    #[serde(rename = "Subqueries")]
    pub subqueries: ServiceRecordSubqueries,
    #[serde(rename = "TimePlayed")]
    pub time_played: String,
    #[serde(rename = "MatchesCompleted")]
    pub matches_completed: i64,
    #[serde(rename = "Wins")]
    pub wins: i64,
    #[serde(rename = "Losses")]
    pub losses: i64,
    #[serde(rename = "Ties")]
    pub ties: i64,
    #[serde(rename = "CoreStats")]
    pub core_stats: CoreStats,
    #[serde(rename = "BombStats")]
    pub bomb_stats: Option<BombStats>,
    #[serde(rename = "CaptureTheFlagStats")]
    pub capture_the_flag_stats: Option<CaptureTheFlagStats>,
    #[serde(rename = "EliminationStats")]
    pub elimination_stats: Option<EliminationStats>,
    #[serde(rename = "ExtractionStats")]
    pub extraction_stats: Option<ExtractionStats>,
    #[serde(rename = "InfectionStats")]
    pub infection_stats: Option<InfectionStats>,
    #[serde(rename = "OddballStats")]
    pub oddball_stats: Option<OddballStats>,
    /// Aggregated Zones stats, using `Zone`-prefixed field names (see [`ServiceRecordZonesStats`]).
    #[serde(rename = "ZonesStats")]
    pub zones_stats: Option<ServiceRecordZonesStats>,
    #[serde(rename = "StockpileStats")]
    pub stockpile_stats: Option<StockpileStats>,
    #[serde(rename = "PvpStats")]
    pub pvp_stats: Option<PvpStats>,
    #[serde(rename = "PveStats")]
    pub pve_stats: Option<PveStats>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ServiceRecordSubqueries {
    #[serde(
        default,
        rename = "SeasonIds",
        deserialize_with = "deserialize_null_default"
    )]
    pub season_ids: Vec<String>,
    #[serde(
        default,
        rename = "GameVariantCategories",
        deserialize_with = "deserialize_null_default"
    )]
    pub game_variant_categories: Vec<i32>,
    #[serde(
        default,
        rename = "IsRanked",
        deserialize_with = "deserialize_null_default"
    )]
    pub is_ranked: Vec<bool>,
    #[serde(
        default,
        rename = "PlaylistAssetIds",
        deserialize_with = "deserialize_null_default"
    )]
    pub playlist_asset_ids: Vec<String>,
    #[serde(rename = "GameplayInteractions")]
    pub gameplay_interactions: serde_json::Value,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CoreStats {
    #[serde(rename = "Score")]
    pub score: i64,
    #[serde(rename = "PersonalScore")]
    pub personal_score: i64,
    #[serde(rename = "RoundsWon")]
    pub rounds_won: i64,
    #[serde(rename = "RoundsLost")]
    pub rounds_lost: i64,
    #[serde(rename = "RoundsTied")]
    pub rounds_tied: i64,
    #[serde(rename = "Kills")]
    pub kills: i64,
    #[serde(rename = "Deaths")]
    pub deaths: i64,
    #[serde(rename = "Assists")]
    pub assists: i64,
    #[serde(rename = "AverageKDA")]
    pub kda: f64,
    #[serde(rename = "Suicides")]
    pub suicides: i64,
    #[serde(rename = "Betrayals")]
    pub betrayals: i64,
    #[serde(rename = "GrenadeKills")]
    pub grenade_kills: i64,
    #[serde(rename = "HeadshotKills")]
    pub headshot_kills: i64,
    #[serde(rename = "MeleeKills")]
    pub melee_kills: i64,
    #[serde(rename = "PowerWeaponKills")]
    pub power_weapon_kills: i64,
    #[serde(rename = "ShotsFired")]
    pub shots_fired: i64,
    #[serde(rename = "ShotsHit")]
    pub shots_hit: i64,
    #[serde(rename = "Accuracy")]
    pub accuracy: f64,
    #[serde(rename = "DamageDealt")]
    pub damage_dealt: i64,
    #[serde(rename = "DamageTaken")]
    pub damage_taken: i64,
    #[serde(rename = "CalloutAssists")]
    pub callout_assists: i64,
    #[serde(rename = "VehicleDestroys")]
    pub vehicle_destroys: i64,
    #[serde(rename = "DriverAssists")]
    pub driver_assists: i64,
    #[serde(rename = "Hijacks")]
    pub hijacks: i64,
    #[serde(rename = "EmpAssists")]
    pub emp_assists: i64,
    #[serde(rename = "MaxKillingSpree")]
    pub max_killing_spree: i64,
    #[serde(rename = "Medals")]
    pub medals: Vec<StatAward>,
    #[serde(rename = "PersonalScores")]
    pub personal_scores: Vec<StatAward>,
    #[serde(rename = "Spawns")]
    pub spawns: i64,
    #[serde(rename = "ObjectivesCompleted")]
    pub objectives_completed: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct StatAward {
    #[serde(rename = "NameId")]
    pub name_id: i64,
    #[serde(rename = "Count")]
    pub count: i64,
    #[serde(rename = "TotalPersonalScoreAwarded")]
    pub total_personal_score_awarded: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserInfo {
    pub xuid: String,
    pub gamertag: String,
    pub gamerpic: Gamerpic,
}

/// Response from the authenticated `/users/me` lookup on `comms.svc.halowaypoint.com`.
///
/// Only `xuid` is confirmed by community reverse-engineering for this specific authority — it is
/// a sibling of, but distinct from, `profile.svc.halowaypoint.com`'s own `/users/{xuid}` lookup
/// (typed as [`UserInfo`]). Other fields this endpoint may carry (gamertag, gamerpic) have not
/// been independently verified here, so they are intentionally omitted rather than guessed; use
/// [`HaloInfiniteClient::user`](crate::clients::hi::HaloInfiniteClient::user) if a gamertag or
/// gamerpic is needed.
#[derive(Debug, Clone, Deserialize)]
pub struct CurrentUser {
    pub xuid: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Gamerpic {
    pub small: String,
    pub medium: String,
    pub large: String,
    pub xlarge: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppearanceCustomization {
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Appearance")]
    pub appearance: PlayerAppearance,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerCustomizationCollection {
    #[serde(rename = "PlayerCustomizations")]
    pub player_customizations: Vec<PlayerCustomization>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerCustomization {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "ResultCode")]
    pub result_code: String,
    #[serde(rename = "Result")]
    pub result: PlayerCustomizationData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerCustomizationData {
    #[serde(rename = "Appearance")]
    pub appearance: PlayerAppearance,
    #[serde(default, rename = "AiCores")]
    pub ai_cores: AiCoreCollection,
    #[serde(default, rename = "ArmorCores")]
    pub armor_cores: ArmorCoreCollection,
    #[serde(default, rename = "VehicleCores")]
    pub vehicle_cores: VehicleCoreCollection,
    #[serde(default, rename = "WeaponCores")]
    pub weapon_cores: WeaponCoreCollection,
    #[serde(default, rename = "SpartanBody")]
    pub spartan_body: Option<SpartanBody>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AiCoreCollection {
    #[serde(default, rename = "AiCores")]
    pub cores: Vec<AiCore>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AiCore {
    pub core_id: String,
    pub core_path: String,
    pub core_type: String,
    pub is_equipped: bool,
    pub themes: Vec<AiTheme>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AiTheme {
    pub color_path: String,
    pub model_path: String,
    pub theme_path: String,
    pub first_modified_date_utc: ApiDate,
    pub last_modified_date_utc: ApiDate,
    pub is_default: bool,
    pub is_equipped: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArmorCoreCollection {
    #[serde(default, rename = "ArmorCores")]
    pub cores: Vec<ArmorCore>,
}

impl ArmorCoreCollection {
    pub fn equipped(&self) -> Option<&ArmorCore> {
        self.cores.iter().find(|core| core.is_equipped)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ArmorCore {
    pub core_id: String,
    pub core_path: String,
    pub core_type: String,
    pub is_equipped: bool,
    pub themes: Vec<ArmorTheme>,
}

impl ArmorCore {
    pub fn equipped_theme(&self) -> Option<&ArmorTheme> {
        self.themes.iter().find(|theme| theme.is_equipped)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ArmorTheme {
    pub armor_fx_path: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub armor_fx_paths: Vec<String>,
    pub chest_attachment_path: String,
    pub coating_path: String,
    #[serde(default)]
    pub emblems: Vec<CustomizationEmblem>,
    pub glove_path: String,
    pub helmet_attachment_path: String,
    pub helmet_path: String,
    pub hip_attachment_path: String,
    pub knee_pad_path: String,
    pub left_shoulder_pad_path: String,
    pub mythic_fx_path: String,
    pub right_shoulder_pad_path: String,
    pub theme_path: String,
    pub visor_path: String,
    pub wrist_attachment_path: String,
    pub first_modified_date_utc: ApiDate,
    pub last_modified_date_utc: ApiDate,
    pub is_default: bool,
    pub is_equipped: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct VehicleCoreCollection {
    #[serde(default, rename = "VehicleCores")]
    pub cores: Vec<VehicleCore>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VehicleCore {
    pub core_id: String,
    pub core_path: String,
    pub core_type: String,
    pub themes: Vec<VehicleTheme>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VehicleTheme {
    pub alternate_geometry_region_path: String,
    pub coating_path: String,
    #[serde(default)]
    pub emblems: Vec<CustomizationEmblem>,
    pub horn_path: String,
    pub theme_path: String,
    pub vehicle_charm_path: String,
    pub vehicle_fx_path: String,
    pub first_modified_date_utc: ApiDate,
    pub last_modified_date_utc: ApiDate,
    pub is_default: bool,
    pub is_equipped: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponCoreCollection {
    #[serde(default, rename = "WeaponCores")]
    pub cores: Vec<WeaponCore>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WeaponCore {
    pub core_id: String,
    pub core_path: String,
    pub core_type: String,
    pub themes: Vec<WeaponTheme>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WeaponTheme {
    pub alternate_geometry_region_path: String,
    pub ammo_counter_color_path: String,
    pub coating_path: String,
    pub death_fx_path: String,
    #[serde(default)]
    pub emblems: Vec<CustomizationEmblem>,
    pub stat_tracker_path: String,
    pub theme_path: String,
    pub weapon_charm_path: String,
    pub first_modified_date_utc: ApiDate,
    pub last_modified_date_utc: ApiDate,
    pub is_default: bool,
    pub is_equipped: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SpartanBody {
    pub body_type: String,
    pub voice_path: String,
    pub left_arm: String,
    pub right_arm: String,
    pub left_leg: String,
    pub right_leg: String,
    pub last_modified_date_utc: ApiDate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerAppearance {
    #[serde(rename = "LastModifiedDateUtc")]
    pub last_modified: Option<ApiDate>,
    #[serde(rename = "ActionPosePath")]
    pub action_pose_path: Option<String>,
    #[serde(rename = "StancePath")]
    pub stance_path: Option<String>,
    #[serde(rename = "BackdropImagePath")]
    pub backdrop_image_path: Option<String>,
    #[serde(rename = "Emblem")]
    pub emblem: Option<EmblemConfiguration>,
    #[serde(rename = "ServiceTag")]
    pub service_tag: String,
    #[serde(rename = "IntroEmotePath")]
    pub intro_emote_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmblemConfiguration {
    #[serde(rename = "EmblemPath")]
    pub emblem_path: String,
    #[serde(rename = "ConfigurationId")]
    pub configuration_id: i64,
}

/// An emblem applied to an armor, weapon, or vehicle location.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CustomizationEmblem {
    pub path: String,
    pub location_id: i64,
    pub configuration_id: i64,
}

impl CustomizationEmblem {
    pub fn emblem_id(&self) -> Option<&str> {
        self.path
            .rsplit('/')
            .next()
            .and_then(|file_name| file_name.strip_suffix(".json"))
    }
}

impl EmblemConfiguration {
    /// Returns the emblem identifier used by [`EmblemMapping`].
    pub fn emblem_id(&self) -> Option<&str> {
        self.emblem_path
            .rsplit('/')
            .next()
            .and_then(|file_name| file_name.strip_suffix(".json"))
    }
}

/// Waypoint image assets indexed by emblem identifier and configuration ID.
#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct EmblemMapping {
    pub emblems: BTreeMap<String, BTreeMap<i64, EmblemImageAssets>>,
}

impl EmblemMapping {
    pub fn get(&self, emblem_id: &str, configuration_id: i64) -> Option<&EmblemImageAssets> {
        self.emblems.get(emblem_id)?.get(&configuration_id)
    }

    /// Resolves an equipped emblem configuration to its display assets.
    pub fn resolve(&self, configuration: &EmblemConfiguration) -> Option<&EmblemImageAssets> {
        self.get(configuration.emblem_id()?, configuration.configuration_id)
    }

    /// Resolves legacy emblem paths through their inventory display metadata.
    pub fn resolve_metadata(
        &self,
        metadata: &CustomizationItemMetadata,
        configuration_id: i64,
    ) -> Option<&EmblemImageAssets> {
        self.get(metadata.image_id()?, configuration_id)
    }

    /// Resolves an equipped emblem configuration's display assets, falling back to *any other*
    /// mapped configuration of the same emblem if the exact equipped color variant isn't mapped.
    ///
    /// Halo's curated mapping table does not necessarily have an entry for every color variant
    /// of every emblem (observed for several players' equipped configurations, not just negative
    /// "Test" palettes), so this tries every configuration actually present in the mapping for
    /// this emblem, preferring [`CustomizationItemMetadata::first_positive_configuration_id`]
    /// first since that ID is most likely to also be the mapping's canonical entry.
    ///
    /// The fallback may not match the player's exact equipped color palette, but resolves to a
    /// real image rather than nothing. Returns `None` only if the emblem has no mapped
    /// configuration at all.
    pub fn resolve_with_fallback(
        &self,
        configuration: &EmblemConfiguration,
        metadata: &CustomizationItemMetadata,
    ) -> Option<&EmblemImageAssets> {
        self.resolve(configuration).or_else(|| {
            let emblem_id = configuration.emblem_id()?;
            let mapped_configurations = self.emblems.get(emblem_id)?;
            metadata
                .first_positive_configuration_id()
                .and_then(|id| mapped_configurations.get(&id))
                .or_else(|| mapped_configurations.values().next())
        })
    }

    pub fn resolve_customization_emblem(
        &self,
        emblem: &CustomizationEmblem,
    ) -> Option<&EmblemImageAssets> {
        self.get(emblem.emblem_id()?, emblem.configuration_id)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmblemImageAssets {
    pub emblem_cms_path: String,
    pub nameplate_cms_path: String,
    pub text_color: String,
}

/// Display metadata for a customization inventory item or core.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomizationItemMetadata {
    #[serde(rename = "CommonData")]
    pub common_data: CustomizationItemCommonData,
    /// Configuration variants available for this item (emblems only). Some emblems' *equipped*
    /// configuration ID is negative — an internal "Test" palette with no image on the CDN — even
    /// though the item itself has other, positive configuration IDs that do resolve to real
    /// images. See [`Self::first_positive_configuration_id`].
    #[serde(default, rename = "AvailableConfigurations")]
    pub available_configurations: Vec<CustomizationConfiguration>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomizationConfiguration {
    #[serde(rename = "ConfigurationId")]
    pub configuration_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomizationItemCommonData {
    #[serde(rename = "Title")]
    pub title: LocalizedText,
    #[serde(rename = "DisplayPath")]
    pub display_path: Option<CustomizationDisplayPath>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomizationDisplayPath {
    #[serde(rename = "Media")]
    pub media: CustomizationMedia,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomizationMedia {
    #[serde(rename = "MediaUrl")]
    pub media_url: CustomizationMediaUrl,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomizationMediaUrl {
    #[serde(rename = "Path")]
    pub path: String,
}

impl CustomizationItemMetadata {
    /// Returns the Game CMS path for this item's display image, when one is defined.
    pub fn image_cms_path(&self) -> Option<&str> {
        self.common_data
            .display_path
            .as_ref()
            .map(|display| display.media.media_url.path.as_str())
            .filter(|path| !path.is_empty())
    }

    /// Returns the display image's file stem, which is also the emblem mapping identifier.
    pub fn image_id(&self) -> Option<&str> {
        self.image_cms_path()?
            .rsplit('/')
            .next()?
            .rsplit_once('.')
            .map(|(stem, _)| stem)
    }

    /// Returns the first positive configuration ID among [`Self::available_configurations`].
    ///
    /// Negative configuration IDs are internal "Test" palettes with no image published to the
    /// CDN. When an [`EmblemMapping`] lookup misses for the player's actual (possibly negative)
    /// equipped configuration, this gives a usable fallback configuration ID for the same
    /// emblem — the resulting image may not match the player's exact equipped color palette, but
    /// it will load, rather than the request 404ing.
    pub fn first_positive_configuration_id(&self) -> Option<i64> {
        self.available_configurations
            .iter()
            .map(|configuration| configuration.configuration_id)
            .find(|id| *id > 0)
    }
}

pub type EmblemMetadata = CustomizationItemMetadata;

impl EmblemImageAssets {
    /// Returns the authenticated Waypoint endpoint for the emblem image.
    ///
    /// Opening this URL without Halo authorization and clearance headers returns HTTP 401.
    pub fn emblem_url(&self) -> String {
        waypoint_file_url(&self.emblem_cms_path)
    }

    /// Returns the authenticated Waypoint endpoint for the nameplate image.
    ///
    /// Opening this URL without Halo authorization and clearance headers returns HTTP 401.
    pub fn nameplate_url(&self) -> String {
        waypoint_file_url(&self.nameplate_cms_path)
    }
}

fn waypoint_file_url(path: &str) -> String {
    format!(
        "{}/{}",
        WAYPOINT_FILE_BASE_URL.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

#[derive(Debug, Clone, Deserialize)]
pub struct BanSummary {
    #[serde(rename = "Results")]
    pub results: Vec<BanSummaryResult>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BanSummaryResult {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "ResultCode")]
    pub result_code: i32,
    #[serde(rename = "Result")]
    pub result: BanResult,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BanResult {
    #[serde(rename = "BansInEffect")]
    pub bans_in_effect: Vec<BanInEffect>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BanInEffect {
    #[serde(rename = "Type")]
    pub ban_type: i32,
    #[serde(rename = "Scope")]
    pub scope: i32,
    #[serde(rename = "EnforceUntilUtc")]
    pub enforce_until: ApiDate,
    #[serde(rename = "BanMessagePath")]
    pub message_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BanMessage {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "DisplayMessage")]
    pub display_message: LocalizedText,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocalizedText {
    pub status: String,
    pub value: String,
    pub translations: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsrSeasonCalendar {
    #[serde(rename = "Seasons")]
    pub seasons: Vec<CsrSeason>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeasonCalendar {
    #[serde(rename = "Seasons")]
    pub seasons: Vec<Season>,
    #[serde(rename = "Events")]
    pub events: Vec<SeasonEvent>,
    #[serde(rename = "CareerRank")]
    pub career_rank: CareerRank,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Season {
    #[serde(rename = "CsrSeasonFilePath")]
    pub csr_season_file_path: String,
    #[serde(rename = "OperationTrackPath")]
    pub operation_track_path: String,
    #[serde(rename = "SeasonMetadata")]
    pub season_metadata: String,
    #[serde(rename = "StartDate")]
    pub start_date: ApiDate,
    #[serde(rename = "EndDate")]
    pub end_date: ApiDate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeasonEvent {
    #[serde(rename = "RewardTrackPath")]
    pub reward_track_path: String,
    #[serde(rename = "StartDate")]
    pub start_date: ApiDate,
    #[serde(rename = "EndDate")]
    pub end_date: ApiDate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CareerRank {
    #[serde(rename = "RewardTrackPath")]
    pub reward_track_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsrSeason {
    #[serde(rename = "CsrSeasonFilePath")]
    pub csr_season_file_path: String,
    #[serde(rename = "StartDate")]
    pub start_date: ApiDate,
    #[serde(rename = "EndDate")]
    pub end_date: ApiDate,
}

impl CsrSeasonCalendar {
    pub fn current(&self, at: DateTime<Utc>) -> Option<&CsrSeason> {
        self.seasons
            .iter()
            .filter(|season| {
                season.start_date.iso8601_date <= at && at < season.end_date.iso8601_date
            })
            .max_by_key(|season| season.start_date.iso8601_date)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiDate {
    #[serde(rename = "ISO8601Date")]
    pub iso8601_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistMetadata {
    #[serde(rename = "NameHint")]
    pub name_hint: String,
    #[serde(rename = "UgcPlaylistVersion")]
    pub ugc_playlist_version: String,
    #[serde(rename = "HasCsr")]
    pub has_csr: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetLink {
    #[serde(rename = "AssetId")]
    pub asset_id: String,
    #[serde(rename = "VersionId")]
    pub version_id: String,
    #[serde(rename = "PublicName")]
    pub public_name: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Files")]
    pub files: Option<AssetFiles>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetFiles {
    #[serde(rename = "Prefix")]
    pub prefix: String,
    #[serde(rename = "FileRelativePaths")]
    pub relative_paths: Vec<String>,
}

impl AssetFiles {
    pub fn url(&self, relative_path: &str) -> String {
        format!(
            "{}/{}",
            self.prefix.trim_end_matches('/'),
            relative_path.trim_start_matches('/')
        )
    }

    pub fn image_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.relative_paths
            .iter()
            .filter(|path| {
                let path = path.to_ascii_lowercase();
                path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg")
            })
            .map(|path| self.url(path))
    }

    /// Finds an image file by name stem (e.g. `"thumbnail"`), regardless of its extension.
    ///
    /// Halo serves the same conventionally-named image (`hero`, `thumbnail`) under different
    /// extensions across assets — some `.png`, others `.jpg` or `.jpeg` — so matching only a
    /// single hardcoded extension misses real images that exist under a different one.
    fn named_image_url(&self, stem: &str) -> Option<String> {
        self.relative_paths
            .iter()
            .find(|path| {
                path.rsplit('/').next().is_some_and(|name| {
                    name.rsplit_once('.').is_some_and(|(name_stem, extension)| {
                        name_stem.eq_ignore_ascii_case(stem)
                            && matches!(
                                extension.to_ascii_lowercase().as_str(),
                                "png" | "jpg" | "jpeg"
                            )
                    })
                })
            })
            .map(|path| self.url(path))
    }

    fn screenshot_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.relative_paths
            .iter()
            .filter(|path| {
                let Some(name) = path.rsplit('/').next() else {
                    return false;
                };
                let name = name.to_ascii_lowercase();
                name.starts_with("screenshot")
                    && (name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg"))
            })
            .map(|path| self.url(path))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistAsset {
    #[serde(flatten)]
    pub asset: AssetLink,
    #[serde(rename = "RotationEntries")]
    pub rotation_entries: Vec<RotationEntry>,
}

impl PlaylistAsset {
    pub fn hero_url(&self) -> Option<String> {
        self.asset
            .files
            .as_ref()
            .and_then(|files| files.named_image_url("hero"))
    }

    pub fn thumbnail_url(&self) -> Option<String> {
        self.asset
            .files
            .as_ref()
            .and_then(|files| files.named_image_url("thumbnail"))
    }

    pub fn screenshot_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.asset
            .files
            .iter()
            .flat_map(AssetFiles::screenshot_urls)
    }

    pub fn image_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.asset.files.iter().flat_map(AssetFiles::image_urls)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RotationEntry {
    #[serde(flatten)]
    pub asset: AssetLink,
    #[serde(rename = "Metadata")]
    pub metadata: RotationMetadata,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RotationMetadata {
    #[serde(rename = "Weight")]
    pub weight: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapModePairAsset {
    #[serde(flatten)]
    pub asset: AssetLink,
    #[serde(rename = "MapLink")]
    pub map: AssetLink,
    #[serde(rename = "UgcGameVariantLink")]
    pub mode: AssetLink,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapAsset {
    #[serde(flatten)]
    pub asset: AssetLink,
    #[serde(rename = "CustomData")]
    pub custom_data: MapCustomData,
}

impl MapAsset {
    pub fn hero_url(&self) -> Option<String> {
        self.asset
            .files
            .as_ref()
            .and_then(|files| files.named_image_url("hero"))
    }

    pub fn thumbnail_url(&self) -> Option<String> {
        self.asset
            .files
            .as_ref()
            .and_then(|files| files.named_image_url("thumbnail"))
    }

    pub fn screenshot_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.asset
            .files
            .iter()
            .flat_map(AssetFiles::screenshot_urls)
    }

    pub fn image_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.asset.files.iter().flat_map(AssetFiles::image_urls)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapCustomData {
    #[serde(rename = "NumOfObjectsOnMap")]
    pub object_count: i64,
    #[serde(rename = "TagLevelId")]
    pub tag_level_id: i64,
    #[serde(rename = "IsBaked")]
    pub is_baked: bool,
    #[serde(rename = "HasNodeGraph")]
    pub has_node_graph: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GameVariantAsset {
    #[serde(flatten)]
    pub asset: AssetLink,
    #[serde(rename = "CustomData")]
    pub custom_data: GameVariantCustomData,
}

impl GameVariantAsset {
    pub fn hero_url(&self) -> Option<String> {
        self.asset
            .files
            .as_ref()
            .and_then(|files| files.named_image_url("hero"))
    }

    pub fn thumbnail_url(&self) -> Option<String> {
        self.asset
            .files
            .as_ref()
            .and_then(|files| files.named_image_url("thumbnail"))
    }

    pub fn screenshot_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.asset
            .files
            .iter()
            .flat_map(AssetFiles::screenshot_urls)
    }

    pub fn image_urls(&self) -> impl Iterator<Item = String> + '_ {
        self.asset.files.iter().flat_map(AssetFiles::image_urls)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GameVariantCustomData {
    #[serde(rename = "KeyValues")]
    pub key_values: serde_json::Value,
    #[serde(rename = "HasNodeGraph")]
    pub has_node_graph: bool,
}

#[derive(Debug, Clone)]
pub struct RankedArenaMapMode {
    pub weight: f64,
    pub pair: MapModePairAsset,
    pub map: MapAsset,
    pub mode: GameVariantAsset,
}

#[derive(Debug, Clone)]
pub struct RankedArenaSeason {
    pub season: CsrSeason,
    pub map_modes: Vec<RankedArenaMapMode>,
}

/// Which pool of games a service-record query aggregates over.
///
/// Filters (see [`ServiceRecordFilter`]) apply only to [`MatchType::Matchmade`]; Halo ignores them
/// for custom and local records.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MatchType {
    #[default]
    Matchmade,
    Custom,
    Local,
}

impl MatchType {
    /// The path segment Halo expects, e.g. `Matchmade` in `.../{match_type}/servicerecord`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Matchmade => "Matchmade",
            Self::Custom => "Custom",
            Self::Local => "Local",
        }
    }
}

/// Which games a match-history query returns.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MatchHistoryType {
    #[default]
    All,
    Matchmaking,
    Custom,
    Local,
}

impl MatchHistoryType {
    /// The value sent in the `type` query parameter.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Matchmaking => "matchmaking",
            Self::Custom => "custom",
            Self::Local => "local",
        }
    }
}

/// Optional filters narrowing a matchmade service record to a season, playlist, or mode.
///
/// Halo only accepts specific combinations of these filters. The most common are a season on its
/// own, or a season plus a game-variant category. Build one fluently:
///
/// ```
/// use halo_api::clients::hi::models::ServiceRecordFilter;
///
/// let filter = ServiceRecordFilter::for_season("Csr/Seasons/CsrSeason5-1.json")
///     .game_variant_category(6)
///     .ranked(true);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServiceRecordFilter {
    pub season_id: Option<String>,
    pub game_variant_category: Option<i32>,
    pub playlist_asset_id: Option<String>,
    pub is_ranked: Option<bool>,
    pub gameplay_interaction: Option<String>,
}

impl ServiceRecordFilter {
    /// Starts a filter scoped to a CSR season file path.
    pub fn for_season(season_id: impl Into<String>) -> Self {
        Self {
            season_id: Some(season_id.into()),
            ..Self::default()
        }
    }

    /// Sets the season file path (e.g. `Csr/Seasons/CsrSeason5-1.json`).
    pub fn season(mut self, season_id: impl Into<String>) -> Self {
        self.season_id = Some(season_id.into());
        self
    }

    /// Restricts to a game-variant category (the numeric `GameVariantCategory`).
    pub fn game_variant_category(mut self, category: i32) -> Self {
        self.game_variant_category = Some(category);
        self
    }

    /// Restricts to a single playlist asset ID.
    pub fn playlist_asset_id(mut self, playlist_asset_id: impl Into<String>) -> Self {
        self.playlist_asset_id = Some(playlist_asset_id.into());
        self
    }

    /// Restricts to ranked (`true`) or social (`false`) results.
    pub fn ranked(mut self, is_ranked: bool) -> Self {
        self.is_ranked = Some(is_ranked);
        self
    }

    /// Restricts to a gameplay-interaction value (e.g. `PvP`, `PvE`).
    pub fn gameplay_interaction(mut self, gameplay_interaction: impl Into<String>) -> Self {
        self.gameplay_interaction = Some(gameplay_interaction.into());
        self
    }

    /// Returns `true` when no filter is set.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Renders the filter as Halo's query parameters (lowercase, no separators).
    pub(crate) fn to_query(&self) -> Vec<(&'static str, String)> {
        let mut query = Vec::new();
        if let Some(season_id) = &self.season_id {
            query.push(("seasonid", season_id.clone()));
        }
        if let Some(category) = self.game_variant_category {
            query.push(("gamevariantcategory", category.to_string()));
        }
        if let Some(playlist_asset_id) = &self.playlist_asset_id {
            query.push(("playlistassetid", playlist_asset_id.clone()));
        }
        if let Some(is_ranked) = self.is_ranked {
            // Halo expects Python-style capitalized booleans here.
            query.push((
                "isranked",
                if is_ranked { "True" } else { "False" }.to_string(),
            ));
        }
        if let Some(gameplay_interaction) = &self.gameplay_interaction {
            query.push(("gameplayinteraction", gameplay_interaction.clone()));
        }
        query
    }
}

/// Match counts across the different game experiences for a player.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MatchCount {
    #[serde(rename = "CustomMatchesPlayedCount")]
    pub custom: i64,
    #[serde(rename = "MatchesPlayedCount")]
    pub total: i64,
    #[serde(rename = "MatchmadeMatchesPlayedCount")]
    pub matchmade: i64,
    #[serde(rename = "LocalMatchesPlayedCount")]
    pub local: i64,
}

/// A player's progress on a reward track (career rank or operation pass).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RewardTrackProgress {
    #[serde(rename = "Rank")]
    pub rank: i32,
    #[serde(rename = "PartialProgress")]
    pub partial_progress: i64,
    #[serde(rename = "HasReachedMaxRank")]
    pub has_reached_max_rank: bool,
}

/// A player's current career-rank progression.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PlayerCareerRank {
    #[serde(rename = "CurrentProgress")]
    pub current_progress: RewardTrackProgress,
    #[serde(rename = "SpartanId")]
    pub spartan_id: Option<String>,
}

/// Response body from the batch career-rank endpoint, keyed by player.
///
/// Unlike [`HaloInfiniteClient::career_rank`](crate::HaloInfiniteClient::career_rank), this
/// endpoint works for any player, not just the authenticated one.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CareerRanks {
    #[serde(rename = "RewardTracks")]
    pub records: Vec<CareerRankRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CareerRankRecord {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "ResultCode")]
    pub result_code: String,
    #[serde(rename = "Result")]
    pub result: PlayerCareerRank,
}

/// A player's owned and available operation passes.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PlayerOperationPasses {
    #[serde(rename = "ActiveOperationRewardTrackPath")]
    pub active_operation_reward_track_path: Option<String>,
    #[serde(rename = "OperationRewardTracks")]
    pub operation_reward_tracks: Vec<PlayerOperationPass>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerOperationPass {
    #[serde(rename = "RewardTrackPath")]
    pub reward_track_path: String,
    #[serde(default, rename = "IsOwned")]
    pub is_owned: bool,
    #[serde(default, rename = "CurrentProgress")]
    pub current_progress: RewardTrackProgress,
}

/// A player's active, upcoming, and completed challenge decks.
///
/// Field names confirmed by cross-checking two independent community reverse-engineerings
/// (OpenSpartan/grunt and SpartanReport) that agree on this shape.
#[derive(Debug, Clone, Deserialize)]
pub struct PlayerChallengeDecks {
    #[serde(rename = "AssignedDecks")]
    pub assigned_decks: Vec<ChallengeDeck>,
    #[serde(rename = "ClearanceId")]
    pub clearance_id: String,
    #[serde(rename = "ActiveRewardTrack")]
    pub active_reward_track: ChallengeRewardTrack,
    #[serde(rename = "ScheduledRewardTrack")]
    pub scheduled_reward_track: ChallengeRewardTrack,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChallengeDeck {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Path")]
    pub path: String,
    #[serde(rename = "ActiveChallenges")]
    pub active_challenges: Vec<Challenge>,
    #[serde(rename = "UpcomingChallenges")]
    pub upcoming_challenges: Vec<Challenge>,
    #[serde(rename = "CompletedChallenges")]
    pub completed_challenges: Vec<Challenge>,
    #[serde(rename = "Expiration")]
    pub expiration: ApiDate,
}

/// A single challenge within a [`ChallengeDeck`].
///
/// Fields beyond `path`/`progress`/`id`/`can_reroll` are only populated on upcoming challenges.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Challenge {
    #[serde(rename = "Path")]
    pub path: String,
    #[serde(rename = "Progress")]
    pub progress: i32,
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "CanReroll")]
    pub can_reroll: bool,
    #[serde(rename = "Difficulty")]
    pub difficulty: Option<String>,
    #[serde(rename = "TypeIconPath")]
    pub type_icon_path: Option<String>,
    #[serde(rename = "IsUserEvent")]
    pub is_user_event: Option<bool>,
    #[serde(rename = "Category")]
    pub category: Option<String>,
    #[serde(rename = "Description")]
    pub description: Option<LocalizedText>,
    #[serde(rename = "Title")]
    pub title: Option<LocalizedText>,
    #[serde(rename = "ThresholdForSuccess")]
    pub threshold_for_success: Option<i32>,
    #[serde(rename = "Reward")]
    pub reward: Option<ChallengeReward>,
    #[serde(rename = "SecondaryReward")]
    pub secondary_reward: Option<ChallengeReward>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ChallengeReward {
    #[serde(
        rename = "InventoryItems",
        deserialize_with = "deserialize_null_default"
    )]
    pub inventory_items: Vec<String>,
    #[serde(rename = "SoftExperience")]
    pub soft_experience: i64,
    #[serde(rename = "OperationExperience")]
    pub operation_experience: i64,
}

/// The reward-track summary embedded in [`PlayerChallengeDecks`].
///
/// Distinct from [`RewardTrackProgress`]/[`PlayerOperationPass`] — this endpoint's embedded
/// summary carries extra fields (`track_type`, `previous_progress`, `base_xp`, `boost_xp`) those
/// don't return, so it is not reused as either of them.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ChallengeRewardTrack {
    #[serde(rename = "RewardTrackPath")]
    pub reward_track_path: String,
    #[serde(rename = "TrackType")]
    pub track_type: String,
    #[serde(rename = "CurrentProgress")]
    pub current_progress: i64,
    #[serde(rename = "PreviousProgress")]
    pub previous_progress: i64,
    #[serde(rename = "IsOwned")]
    pub is_owned: bool,
    #[serde(rename = "BaseXp")]
    pub base_xp: i64,
    #[serde(rename = "BoostXp")]
    pub boost_xp: i64,
    #[serde(rename = "HasReachedMaxRank")]
    pub has_reached_max_rank: bool,
}

/// The career-rank reward-track definition from the Game CMS.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CareerRewardTrack {
    #[serde(rename = "TrackId")]
    pub track_id: String,
    #[serde(rename = "XpPerRank")]
    pub xp_per_rank: i64,
    #[serde(rename = "Ranks")]
    pub ranks: Vec<RewardTrackRank>,
}

impl CareerRewardTrack {
    /// Looks up the rank definition (title, tier, icons) for a numeric rank, as reported by
    /// [`RewardTrackProgress::rank`] (via [`PlayerCareerRank`] or [`CareerRankRecord`]).
    pub fn rank(&self, rank: i32) -> Option<&RewardTrackRank> {
        self.ranks.iter().find(|entry| entry.rank == rank)
    }
}

/// An operation (battle pass) reward-track definition from the Game CMS.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct OperationRewardTrack {
    #[serde(rename = "Name")]
    pub name: Option<LocalizedText>,
    #[serde(rename = "Description")]
    pub description: Option<LocalizedText>,
    #[serde(rename = "XpPerRank")]
    pub xp_per_rank: i64,
    #[serde(rename = "Ranks")]
    pub ranks: Vec<RewardTrackRank>,
}

/// One rank tier within a reward track, with the rewards granted at that rank.
///
/// For the career-rank track specifically, this also carries the rank's display title, tier,
/// and icon paths (e.g. "Bronze Cadet 1"), which [`PlayerCareerRank::current_progress`]'s numeric
/// rank does not include on its own.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RewardTrackRank {
    #[serde(rename = "Rank")]
    pub rank: i32,
    #[serde(rename = "XpRequiredForRank")]
    pub xp_required_for_rank: i64,
    #[serde(rename = "FreeRewards")]
    pub free_rewards: RankRewards,
    #[serde(rename = "PaidRewards")]
    pub paid_rewards: RankRewards,
    /// The rank's title, e.g. "Cadet" (career rank only).
    #[serde(rename = "RankTitle")]
    pub rank_title: Option<LocalizedText>,
    /// The rank's grade within its title, e.g. "1" (career rank only).
    #[serde(rename = "RankSubTitle")]
    pub rank_sub_title: Option<LocalizedText>,
    /// The rank's tier name, e.g. "Bronze" (career rank only).
    #[serde(rename = "RankTier")]
    pub rank_tier: Option<LocalizedText>,
    /// The rank's tier identifier, e.g. "TierBronze" (career rank only).
    #[serde(rename = "TierType")]
    pub tier_type: Option<String>,
    /// CMS path to the small rank icon (career rank only).
    #[serde(rename = "RankIcon")]
    pub rank_icon: Option<String>,
    /// CMS path to the large rank icon (career rank only).
    #[serde(rename = "RankLargeIcon")]
    pub rank_large_icon: Option<String>,
    /// CMS path to the rank adornment icon (career rank only).
    #[serde(rename = "RankAdornmentIcon")]
    pub rank_adornment_icon: Option<String>,
    /// The rank's numeric grade within its title, e.g. `1` (career rank only).
    #[serde(rename = "RankGrade")]
    pub rank_grade: Option<i32>,
}

impl RewardTrackRank {
    /// Returns this rank's full display title, e.g. "Cadet 1" (career rank only).
    pub fn display_title(&self) -> Option<String> {
        let title = self.rank_title.as_ref()?.value.as_str();
        match self.rank_sub_title.as_ref().map(|s| s.value.as_str()) {
            Some(sub_title) if !sub_title.is_empty() => Some(format!("{title} {sub_title}")),
            _ => Some(title.to_string()),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RankRewards {
    #[serde(
        rename = "InventoryRewards",
        deserialize_with = "deserialize_null_default"
    )]
    pub inventory_rewards: Vec<InventoryReward>,
    #[serde(
        rename = "CurrencyRewards",
        deserialize_with = "deserialize_null_default"
    )]
    pub currency_rewards: Vec<CurrencyReward>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InventoryReward {
    #[serde(rename = "InventoryItemPath")]
    pub inventory_item_path: String,
    #[serde(default, rename = "Amount")]
    pub amount: i64,
    #[serde(default, rename = "Type")]
    pub reward_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CurrencyReward {
    #[serde(rename = "CurrencyPath")]
    pub currency_path: String,
    #[serde(default, rename = "Amount")]
    pub amount: i64,
}

/// Per-player skill (CSR and MMR) results for a single match.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MatchSkill {
    #[serde(rename = "Value")]
    pub results: Vec<MatchSkillResult>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchSkillResult {
    /// The player this entry describes, in `xuid(...)` form.
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "ResultCode")]
    pub result_code: i32,
    #[serde(rename = "Result")]
    pub result: MatchSkillDetail,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MatchSkillDetail {
    #[serde(rename = "TeamMmr")]
    pub team_mmr: f64,
    #[serde(rename = "TeamId")]
    pub team_id: i32,
    #[serde(rename = "TeamMmrs")]
    pub team_mmrs: BTreeMap<String, f64>,
    #[serde(rename = "RankRecap")]
    pub rank_recap: RankRecap,
    /// Modeled per-player kill/death performance, absent for some matches.
    #[serde(rename = "StatPerformances")]
    pub stat_performances: Option<StatPerformances>,
    /// How the player's kills/deaths compared to expectations, absent for social matches.
    #[serde(rename = "Counterfactuals")]
    pub counterfactuals: Option<Counterfactuals>,
    /// Ranked reward for the match. Halo reports this as `null` in practice.
    #[serde(default, rename = "RankedRewards")]
    pub ranked_rewards: Option<RankedRewards>,
}

/// A player's CSR before and after the match.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RankRecap {
    #[serde(rename = "PreMatchCsr")]
    pub pre_match_csr: CsrRecordRanking,
    #[serde(rename = "PostMatchCsr")]
    pub post_match_csr: CsrRecordRanking,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct StatPerformances {
    #[serde(rename = "Kills")]
    pub kills: StatPerformance,
    #[serde(rename = "Deaths")]
    pub deaths: StatPerformance,
}

/// A player's actual, expected, and variance for a single tracked stat.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct StatPerformance {
    #[serde(rename = "Count")]
    pub count: f64,
    #[serde(rename = "Expected")]
    pub expected: f64,
    #[serde(rename = "StdDev")]
    pub std_dev: f64,
}

/// How a player's kills and deaths compare to skill-model expectations.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Counterfactuals {
    #[serde(rename = "SelfCounterfactuals")]
    pub own: Counterfactual,
    #[serde(rename = "TierCounterfactuals")]
    pub by_tier: BTreeMap<String, Counterfactual>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Counterfactual {
    #[serde(rename = "Kills")]
    pub kills: f64,
    #[serde(rename = "Deaths")]
    pub deaths: f64,
}

/// Mode-specific stat blocks that accompany the core stats for a team or player.
///
/// Every field is optional: Halo only includes the block matching the match's mode. Durations are
/// ISO-8601 duration strings (e.g. `PT10M30S`), matching [`MatchInfo::duration`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ModeStats {
    #[serde(rename = "BombStats")]
    pub bomb: Option<BombStats>,
    #[serde(rename = "CaptureTheFlagStats")]
    pub capture_the_flag: Option<CaptureTheFlagStats>,
    #[serde(rename = "EliminationStats")]
    pub elimination: Option<EliminationStats>,
    #[serde(rename = "ExtractionStats")]
    pub extraction: Option<ExtractionStats>,
    #[serde(rename = "InfectionStats")]
    pub infection: Option<InfectionStats>,
    #[serde(rename = "OddballStats")]
    pub oddball: Option<OddballStats>,
    /// Per-match Zones (Strongholds) stats, using `Stronghold`-prefixed field names.
    #[serde(rename = "ZonesStats")]
    pub zones: Option<ZonesStats>,
    #[serde(rename = "StockpileStats")]
    pub stockpile: Option<StockpileStats>,
    #[serde(rename = "PvpStats")]
    pub pvp: Option<PvpStats>,
    #[serde(rename = "PveStats")]
    pub pve: Option<PveStats>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct BombStats {
    #[serde(rename = "BombCarriersKilled")]
    pub bomb_carriers_killed: i64,
    #[serde(rename = "BombDefusals")]
    pub bomb_defusals: i64,
    #[serde(rename = "BombDefusersKilled")]
    pub bomb_defusers_killed: i64,
    #[serde(rename = "BombDetonations")]
    pub bomb_detonations: i64,
    #[serde(rename = "BombPickUps")]
    pub bomb_pick_ups: i64,
    #[serde(rename = "BombPlants")]
    pub bomb_plants: i64,
    #[serde(rename = "BombReturns")]
    pub bomb_returns: i64,
    #[serde(rename = "KillsAsBombCarrier")]
    pub kills_as_bomb_carrier: i64,
    #[serde(rename = "TimeAsBombCarrier")]
    pub time_as_bomb_carrier: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CaptureTheFlagStats {
    #[serde(rename = "FlagCaptureAssists")]
    pub flag_capture_assists: i64,
    #[serde(rename = "FlagCaptures")]
    pub flag_captures: i64,
    #[serde(rename = "FlagCarriersKilled")]
    pub flag_carriers_killed: i64,
    #[serde(rename = "FlagGrabs")]
    pub flag_grabs: i64,
    #[serde(rename = "FlagReturnersKilled")]
    pub flag_returners_killed: i64,
    #[serde(rename = "FlagReturns")]
    pub flag_returns: i64,
    #[serde(rename = "FlagSecures")]
    pub flag_secures: i64,
    #[serde(rename = "FlagSteals")]
    pub flag_steals: i64,
    #[serde(rename = "KillsAsFlagCarrier")]
    pub kills_as_flag_carrier: i64,
    #[serde(rename = "KillsAsFlagReturner")]
    pub kills_as_flag_returner: i64,
    #[serde(rename = "TimeAsFlagCarrier")]
    pub time_as_flag_carrier: String,
}

/// Elimination stats. The per-match block additionally carries `LivesRemaining` and
/// `EliminationOrder`, which the service-record aggregate omits.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct EliminationStats {
    #[serde(rename = "AlliesRevived")]
    pub allies_revived: i64,
    #[serde(rename = "EliminationAssists")]
    pub elimination_assists: i64,
    #[serde(rename = "Eliminations")]
    pub eliminations: i64,
    #[serde(rename = "EnemyRevivesDenied")]
    pub enemy_revives_denied: i64,
    #[serde(rename = "Executions")]
    pub executions: i64,
    #[serde(rename = "KillsAsLastPlayerStanding")]
    pub kills_as_last_player_standing: i64,
    #[serde(rename = "LastPlayersStandingKilled")]
    pub last_players_standing_killed: i64,
    #[serde(rename = "RoundsSurvived")]
    pub rounds_survived: i64,
    #[serde(rename = "TimesRevivedByAlly")]
    pub times_revived_by_ally: i64,
    #[serde(rename = "LivesRemaining")]
    pub lives_remaining: Option<i64>,
    #[serde(rename = "EliminationOrder")]
    pub elimination_order: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ExtractionStats {
    #[serde(rename = "ExtractionConversionsCompleted")]
    pub extraction_conversions_completed: i64,
    #[serde(rename = "ExtractionConversionsDenied")]
    pub extraction_conversions_denied: i64,
    #[serde(rename = "ExtractionInitiationsCompleted")]
    pub extraction_initiations_completed: i64,
    #[serde(rename = "ExtractionInitiationsDenied")]
    pub extraction_initiations_denied: i64,
    #[serde(rename = "SuccessfulExtractions")]
    pub successful_extractions: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct InfectionStats {
    #[serde(rename = "AlphasKilled")]
    pub alphas_killed: i64,
    #[serde(rename = "InfectedKilled")]
    pub infected_killed: i64,
    #[serde(rename = "KillsAsLastSpartanStanding")]
    pub kills_as_last_spartan_standing: i64,
    #[serde(rename = "LastSpartansStandingInfected")]
    pub last_spartans_standing_infected: i64,
    #[serde(rename = "RoundsAsAlpha")]
    pub rounds_as_alpha: i64,
    #[serde(rename = "RoundsAsLastSpartanStanding")]
    pub rounds_as_last_spartan_standing: i64,
    #[serde(rename = "RoundsFinishedAsInfected")]
    pub rounds_finished_as_infected: i64,
    #[serde(rename = "RoundsSurvivedAsLastSpartanStanding")]
    pub rounds_survived_as_last_spartan_standing: i64,
    #[serde(rename = "RoundsSurvivedAsSpartan")]
    pub rounds_survived_as_spartan: i64,
    #[serde(rename = "SpartansInfected")]
    pub spartans_infected: i64,
    #[serde(rename = "SpartansInfectedAsAlpha")]
    pub spartans_infected_as_alpha: i64,
    #[serde(rename = "TimeAsLastSpartanStanding")]
    pub time_as_last_spartan_standing: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct OddballStats {
    #[serde(rename = "KillsAsSkullCarrier")]
    pub kills_as_skull_carrier: i64,
    #[serde(rename = "LongestTimeAsSkullCarrier")]
    pub longest_time_as_skull_carrier: String,
    #[serde(rename = "SkullCarriersKilled")]
    pub skull_carriers_killed: i64,
    #[serde(rename = "SkullGrabs")]
    pub skull_grabs: i64,
    #[serde(rename = "SkullScoringTicks")]
    pub skull_scoring_ticks: i64,
    #[serde(rename = "TimeAsSkullCarrier")]
    pub time_as_skull_carrier: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct StockpileStats {
    #[serde(rename = "KillsAsPowerSeedCarrier")]
    pub kills_as_power_seed_carrier: i64,
    #[serde(rename = "PowerSeedCarriersKilled")]
    pub power_seed_carriers_killed: i64,
    #[serde(rename = "PowerSeedsDeposited")]
    pub power_seeds_deposited: i64,
    #[serde(rename = "PowerSeedsStolen")]
    pub power_seeds_stolen: i64,
    #[serde(rename = "TimeAsPowerSeedCarrier")]
    pub time_as_power_seed_carrier: String,
    #[serde(rename = "TimeAsPowerSeedDriver")]
    pub time_as_power_seed_driver: String,
}

/// Per-match Zones (Strongholds) stats. The service record aggregates these under
/// [`ServiceRecordZonesStats`] with `Zone`-prefixed field names instead.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ZonesStats {
    #[serde(rename = "StrongholdCaptures")]
    pub stronghold_captures: i64,
    #[serde(rename = "StrongholdDefensiveKills")]
    pub stronghold_defensive_kills: i64,
    #[serde(rename = "StrongholdOffensiveKills")]
    pub stronghold_offensive_kills: i64,
    #[serde(rename = "StrongholdSecures")]
    pub stronghold_secures: i64,
    #[serde(rename = "StrongholdOccupationTime")]
    pub stronghold_occupation_time: String,
    #[serde(rename = "StrongholdScoringTicks")]
    pub stronghold_scoring_ticks: i64,
}

/// Service-record Zones stats, aggregated with `Zone`-prefixed field names (unlike the per-match
/// [`ZonesStats`], which uses `Stronghold` prefixes).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ServiceRecordZonesStats {
    #[serde(rename = "ZoneCaptures")]
    pub zone_captures: i64,
    #[serde(rename = "ZoneDefensiveKills")]
    pub zone_defensive_kills: i64,
    #[serde(rename = "ZoneOffensiveKills")]
    pub zone_offensive_kills: i64,
    #[serde(rename = "ZoneSecures")]
    pub zone_secures: i64,
    #[serde(rename = "TotalZoneOccupationTime")]
    pub total_zone_occupation_time: String,
    #[serde(rename = "ZoneScoringTicks")]
    pub zone_scoring_ticks: i64,
}

/// Player-vs-player kill totals. The per-match block additionally carries `KDA`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PvpStats {
    #[serde(rename = "Kills")]
    pub kills: i64,
    #[serde(rename = "Deaths")]
    pub deaths: i64,
    #[serde(rename = "Assists")]
    pub assists: i64,
    #[serde(rename = "KDA")]
    pub kda: Option<f64>,
}

/// Player-vs-environment kill totals, broken down by enemy type. The per-match block also carries
/// `KDA`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PveStats {
    #[serde(rename = "Kills")]
    pub kills: i64,
    #[serde(rename = "Deaths")]
    pub deaths: i64,
    #[serde(rename = "Assists")]
    pub assists: i64,
    #[serde(rename = "KDA")]
    pub kda: Option<f64>,
    #[serde(rename = "BossKills")]
    pub boss_kills: i64,
    #[serde(rename = "BruteKills")]
    pub brute_kills: i64,
    #[serde(rename = "EliteKills")]
    pub elite_kills: i64,
    #[serde(rename = "GruntKills")]
    pub grunt_kills: i64,
    #[serde(rename = "HunterKills")]
    pub hunter_kills: i64,
    #[serde(rename = "JackalKills")]
    pub jackal_kills: i64,
    #[serde(rename = "MarineKills")]
    pub marine_kills: i64,
    #[serde(rename = "SentinelKills")]
    pub sentinel_kills: i64,
    #[serde(rename = "SkimmerKills")]
    pub skimmer_kills: i64,
}

/// Localized medal metadata catalog from the Game CMS.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MedalMetadata {
    pub difficulties: Vec<String>,
    pub types: Vec<String>,
    pub sprites: MedalSpriteSheets,
    pub medals: Vec<Medal>,
}

impl MedalMetadata {
    /// Looks up a medal definition by its `name_id`.
    pub fn medal(&self, name_id: i64) -> Option<&Medal> {
        self.medals.iter().find(|medal| medal.name_id == name_id)
    }

    /// Resolves a medal's difficulty label (e.g. `legendary`) via its difficulty index.
    pub fn difficulty_of(&self, medal: &Medal) -> Option<&str> {
        self.difficulties
            .get(usize::try_from(medal.difficulty_index).ok()?)
            .map(String::as_str)
    }

    /// Resolves a medal's type label (e.g. `multikill`) via its type index.
    pub fn type_of(&self, medal: &Medal) -> Option<&str> {
        self.types
            .get(usize::try_from(medal.type_index).ok()?)
            .map(String::as_str)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Medal {
    #[serde(rename = "nameId")]
    pub name_id: i64,
    pub name: MedalText,
    pub description: MedalText,
    #[serde(rename = "spriteIndex")]
    pub sprite_index: i64,
    #[serde(rename = "sortingWeight")]
    pub sorting_weight: i64,
    /// Index into [`MedalMetadata::difficulties`].
    #[serde(rename = "difficultyIndex")]
    pub difficulty_index: i64,
    /// Index into [`MedalMetadata::types`].
    #[serde(rename = "typeIndex")]
    pub type_index: i64,
    #[serde(rename = "personalScore")]
    pub personal_score: i64,
}

/// A medal's localized name or description. Unlike [`LocalizedText`], medal strings carry no
/// `status` field.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MedalText {
    pub value: String,
    pub translations: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MedalSpriteSheets {
    pub small: MedalSpriteSheet,
    pub medium: MedalSpriteSheet,
    #[serde(rename = "extra-large")]
    pub extra_large: MedalSpriteSheet,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MedalSpriteSheet {
    pub path: String,
    pub columns: i64,
    pub size: i64,
}

/// Ranked reward granted for a match, when present. Halo reports this as `null` in practice.
#[derive(Debug, Clone, Deserialize)]
pub struct RankedRewards {
    #[serde(rename = "RewardId")]
    pub reward_id: String,
}

/// A player's participation window within a match.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ParticipationInfo {
    #[serde(rename = "FirstJoinedTime")]
    pub first_joined_time: Option<DateTime<Utc>>,
    #[serde(rename = "LastLeaveTime")]
    pub last_leave_time: Option<DateTime<Utc>>,
    #[serde(rename = "PresentAtBeginning")]
    pub present_at_beginning: bool,
    #[serde(rename = "JoinedInProgress")]
    pub joined_in_progress: bool,
    #[serde(rename = "LeftInProgress")]
    pub left_in_progress: bool,
    #[serde(rename = "PresentAtCompletion")]
    pub present_at_completion: bool,
    #[serde(rename = "TimePlayed")]
    pub time_played: String,
    #[serde(rename = "ConfirmedParticipation")]
    pub confirmed_participation: Option<bool>,
}

/// Bot difficulty attributes, present only for bot players.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct BotAttributes {
    #[serde(rename = "Difficulty")]
    pub difficulty: i32,
}

/// Halo's HIPC endpoint-discovery manifest.
///
/// This is the JSON content-negotiated representation of the same manifest
/// `examples/discover_endpoints.rs` parses as XML directly; the fields are isomorphic to that
/// already-known XML shape (same names, JSON key casing preserved).
#[derive(Debug, Clone, Deserialize)]
pub struct HipcSettings {
    #[serde(rename = "Authorities")]
    pub authorities: BTreeMap<String, Authority>,
    #[serde(rename = "RetryPolicies")]
    pub retry_policies: BTreeMap<String, RetryPolicy>,
    #[serde(rename = "Settings")]
    pub settings: BTreeMap<String, String>,
    #[serde(rename = "Endpoints")]
    pub endpoints: BTreeMap<String, Endpoint>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Authority {
    #[serde(rename = "AuthorityId")]
    pub authority_id: String,
    #[serde(rename = "Scheme")]
    pub scheme: i32,
    #[serde(rename = "Hostname")]
    pub hostname: String,
    #[serde(rename = "Port")]
    pub port: i32,
    #[serde(rename = "AuthenticationMethods")]
    pub authentication_methods: Vec<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetryPolicy {
    #[serde(rename = "RetryPolicyId")]
    pub retry_policy_id: String,
    #[serde(rename = "TimeoutMs")]
    pub timeout_ms: i32,
    #[serde(rename = "RetryOptions")]
    pub retry_options: RetryOptions,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetryOptions {
    #[serde(rename = "MaxRetryCount")]
    pub max_retry_count: i32,
    #[serde(rename = "RetryDelayMs")]
    pub retry_delay_ms: i32,
    #[serde(rename = "RetryGrowth")]
    pub retry_growth: f64,
    #[serde(rename = "RetryJitterMs")]
    pub retry_jitter_ms: i32,
    #[serde(rename = "RetryIfNotFound")]
    pub retry_if_not_found: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Endpoint {
    #[serde(rename = "AuthorityId")]
    pub authority_id: String,
    #[serde(rename = "Path")]
    pub path: String,
    #[serde(rename = "QueryString")]
    pub query_string: Option<String>,
    #[serde(rename = "RetryPolicyId")]
    pub retry_policy_id: String,
    #[serde(rename = "TopicName")]
    pub topic_name: Option<String>,
    #[serde(rename = "AcknowledgementTypeId")]
    pub acknowledgement_type_id: i32,
    #[serde(rename = "AuthenticationLifetimeExtensionSupported")]
    pub authentication_lifetime_extension_supported: bool,
    #[serde(rename = "ClearanceAware")]
    pub clearance_aware: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_ids_support_named_and_custom_values() {
        assert_eq!(PlaylistId::RANKED_ARENA.as_str().len(), 36);
        assert_eq!(PlaylistId::new("custom").as_str(), "custom");
        assert_eq!(GameModeId::RANKED_SLAYER.asset_id().len(), 36);
        assert_eq!(GameModeId::new("asset", "version").version_id(), "version");
        assert_eq!(MapId::LIVE_FIRE.asset_id().len(), 36);
        assert_eq!(MapId::new("asset", "version").version_id(), "version");
    }

    #[test]
    fn map_exposes_ugc_image_urls() {
        let map: MapAsset = serde_json::from_value(serde_json::json!({
            "AssetId": "asset",
            "VersionId": "version",
            "PublicName": "Live Fire",
            "Description": "",
            "Files": {
                "Prefix": "https://cdn.example/map/",
                "FileRelativePaths": [
                    "map.mvar",
                    "images/hero.png",
                    "images/screenshot.jpg"
                ]
            },
            "CustomData": {
                "NumOfObjectsOnMap": 0,
                "TagLevelId": 0,
                "IsBaked": true,
                "HasNodeGraph": false
            }
        }))
        .unwrap();

        assert_eq!(
            map.image_urls().collect::<Vec<_>>(),
            [
                "https://cdn.example/map/images/hero.png",
                "https://cdn.example/map/images/screenshot.jpg"
            ]
        );
        assert_eq!(
            map.hero_url().as_deref(),
            Some("https://cdn.example/map/images/hero.png")
        );
        assert!(map.thumbnail_url().is_none());
        assert_eq!(
            map.screenshot_urls().collect::<Vec<_>>(),
            ["https://cdn.example/map/images/screenshot.jpg"]
        );
    }

    #[test]
    fn game_mode_exposes_ugc_image_urls() {
        let mode: GameVariantAsset = serde_json::from_value(serde_json::json!({
            "AssetId": "asset",
            "VersionId": "version",
            "PublicName": "Ranked:Slayer",
            "Description": "",
            "Files": {
                "Prefix": "https://cdn.example/mode/",
                "FileRelativePaths": [
                    "images/hero.png",
                    "images/thumbnail.png",
                    "images/screenshot1.Png"
                ]
            },
            "CustomData": {
                "KeyValues": {},
                "HasNodeGraph": false
            }
        }))
        .unwrap();

        assert_eq!(
            mode.hero_url().as_deref(),
            Some("https://cdn.example/mode/images/hero.png")
        );
        assert_eq!(
            mode.thumbnail_url().as_deref(),
            Some("https://cdn.example/mode/images/thumbnail.png")
        );
        assert_eq!(
            mode.screenshot_urls().collect::<Vec<_>>(),
            ["https://cdn.example/mode/images/screenshot1.Png"]
        );
    }

    #[test]
    fn named_image_url_finds_jpg_and_jpeg_thumbnails_and_heroes() {
        // Halo serves the conventionally-named hero/thumbnail images under different
        // extensions across assets, not just `.png` (observed live for several Ranked Arena
        // rotation maps, e.g. "Solitude - Ranked", which ship `images/thumbnail.jpg`).
        let map: MapAsset = serde_json::from_value(serde_json::json!({
            "AssetId": "asset",
            "VersionId": "version",
            "PublicName": "Solitude",
            "Description": "",
            "Files": {
                "Prefix": "https://cdn.example/map/",
                "FileRelativePaths": [
                    "images/hero.jpeg",
                    "images/thumbnail.jpg"
                ]
            },
            "CustomData": {
                "NumOfObjectsOnMap": 0,
                "TagLevelId": 0,
                "IsBaked": true,
                "HasNodeGraph": false
            }
        }))
        .unwrap();

        assert_eq!(
            map.hero_url().as_deref(),
            Some("https://cdn.example/map/images/hero.jpeg")
        );
        assert_eq!(
            map.thumbnail_url().as_deref(),
            Some("https://cdn.example/map/images/thumbnail.jpg")
        );
    }

    fn ranking(value: i32) -> CsrRecordRanking {
        CsrRecordRanking {
            value,
            tier: "Platinum".to_string(),
            sub_tier: 2,
            ..CsrRecordRanking::default()
        }
    }

    #[test]
    fn negative_one_is_unranked() {
        assert!(ranking(-1).is_unranked());
    }

    #[test]
    fn positive_value_is_ranked() {
        assert!(!ranking(1500).is_unranked());
    }

    #[test]
    fn unranked_uses_unranked_icon() {
        assert_eq!(
            ranking(-1).rank_icon_url(),
            "https://trackercdn.com/cdn/tracker.gg/halo-infinite/skills/c/unranked.png"
        );
    }

    #[test]
    fn named_tier_icon_uses_lowercase_tier_and_one_indexed_subtier() {
        // `ranking()` fixes tier="Platinum", sub_tier=2 (0-indexed) -> "platinum-3.png".
        assert_eq!(
            ranking(1234).rank_icon_url(),
            "https://trackercdn.com/cdn/tracker.gg/halo-infinite/skills/c/platinum-3.png"
        );
    }

    #[test]
    fn onyx_icon_has_no_subtier_suffix() {
        let onyx = CsrRecordRanking {
            value: 1650,
            tier: "Onyx".to_string(),
            sub_tier: 0,
            ..CsrRecordRanking::default()
        };
        assert_eq!(
            onyx.rank_icon_url(),
            "https://trackercdn.com/cdn/tracker.gg/halo-infinite/skills/c/onyx.png"
        );
    }

    #[test]
    fn deserializes_full_csr_response_shape() {
        let json = serde_json::json!({
            "Value": [{
                "Id": "xuid(123)",
                "ResultCode": 0,
                "Result": {
                    "Current": {
                        "Value": 1500,
                        "MeasurementMatchesRemaining": 0,
                        "Tier": "Platinum",
                        "TierStart": 1200,
                        "SubTier": 2,
                        "NextTier": "Diamond",
                        "NextTierStart": 1500,
                        "NextSubTier": 0,
                        "InitialMeasurementMatches": 5,
                        "DemotionProtectionMatchesRemaining": 2,
                        "InitialDemotionProtectionMatches": 3
                    },
                    "SeasonMax": { "Value": 1550, "Tier": "Diamond", "SubTier": 0 },
                    "AllTimeMax": { "Value": 1600, "Tier": "Diamond", "SubTier": 0 },
                }
            }]
        });
        let records: CsrRecords = serde_json::from_value(json).unwrap();
        assert_eq!(records.records.len(), 1);
        assert_eq!(records.records[0].result.current.value, 1500);
        assert_eq!(records.records[0].result.current.tier_start, 1200);
        assert_eq!(records.records[0].result.current.next_tier, "Diamond");
        assert_eq!(
            records.records[0]
                .result
                .current
                .initial_measurement_matches,
            5
        );
        assert_eq!(records.records[0].result.season_max.value, 1550);
        assert_eq!(records.records[0].result.peak.tier, "Diamond");
    }

    #[test]
    fn current_csr_season_requires_time_inside_date_range() {
        let calendar: CsrSeasonCalendar = serde_json::from_value(serde_json::json!({
            "Seasons": [{
                "CsrSeasonFilePath": "past.json",
                "StartDate": { "ISO8601Date": "2026-01-01T00:00:00Z" },
                "EndDate": { "ISO8601Date": "2026-02-01T00:00:00Z" }
            }, {
                "CsrSeasonFilePath": "overlapping-older.json",
                "StartDate": { "ISO8601Date": "2026-02-20T00:00:00Z" },
                "EndDate": { "ISO8601Date": "2026-04-01T00:00:00Z" }
            }, {
                "CsrSeasonFilePath": "current.json",
                "StartDate": { "ISO8601Date": "2026-03-01T00:00:00Z" },
                "EndDate": { "ISO8601Date": "2026-04-01T00:00:00Z" }
            }]
        }))
        .unwrap();

        let during = "2026-03-15T00:00:00Z".parse().unwrap();
        let gap = "2026-02-15T00:00:00Z".parse().unwrap();
        let at_end = "2026-04-01T00:00:00Z".parse().unwrap();

        assert_eq!(
            calendar.current(during).unwrap().csr_season_file_path,
            "current.json"
        );
        assert!(calendar.current(gap).is_none());
        assert!(calendar.current(at_end).is_none());
    }

    #[test]
    fn service_record_filter_renders_supported_query_shape() {
        assert!(ServiceRecordFilter::default().is_empty());
        assert!(ServiceRecordFilter::default().to_query().is_empty());

        let filter = ServiceRecordFilter::for_season("Csr/Seasons/CsrSeason5-1.json")
            .game_variant_category(6)
            .playlist_asset_id("playlist-1")
            .ranked(false)
            .gameplay_interaction("PvP");
        assert!(!filter.is_empty());
        assert_eq!(
            filter.to_query(),
            vec![
                ("seasonid", "Csr/Seasons/CsrSeason5-1.json".to_string()),
                ("gamevariantcategory", "6".to_string()),
                ("playlistassetid", "playlist-1".to_string()),
                ("isranked", "False".to_string()),
                ("gameplayinteraction", "PvP".to_string()),
            ]
        );
    }

    #[test]
    fn per_match_stats_use_typed_mode_blocks() {
        // Shapes captured from live match_stats responses.
        let block: MatchStatsBlock = serde_json::from_value(serde_json::json!({
            "CoreStats": {
                "Score": 100, "PersonalScore": 2500, "Kills": 20, "Deaths": 10, "Assists": 5,
                "KDA": 1.5, "AverageLifeDuration": "PT30S", "Medals": [
                    { "NameId": 622331684, "Count": 1, "TotalPersonalScoreAwarded": 50 }
                ],
                "ObjectivesCompleted": 3, "MaxKillingSpree": 4
            },
            "ZonesStats": {
                "StrongholdCaptures": 6,
                "StrongholdOccupationTime": "PT54.7S",
                "StrongholdScoringTicks": 0
            },
            "PvpStats": { "Kills": 20, "Deaths": 10, "Assists": 5, "KDA": 1.5 }
        }))
        .unwrap();

        assert_eq!(block.core.kills, 20);
        assert_eq!(block.core.max_killing_spree, 4);
        assert_eq!(block.core.objectives_completed, 3);
        assert_eq!(block.core.medals[0].name_id, 622331684);
        // Per-match zones use Stronghold-prefixed names.
        let zones = block.mode_stats.zones.as_ref().unwrap();
        assert_eq!(zones.stronghold_captures, 6);
        assert_eq!(zones.stronghold_occupation_time, "PT54.7S");
        let pvp = block.mode_stats.pvp.as_ref().unwrap();
        assert_eq!(pvp.kills, 20);
        assert_eq!(pvp.kda, Some(1.5));
        assert!(block.mode_stats.oddball.is_none());
    }

    #[test]
    fn service_record_zones_use_zone_prefixed_names() {
        // The service-record aggregate uses Zone-prefixed names, unlike per-match Stronghold ones.
        let record: ServiceRecord = serde_json::from_value(serde_json::json!({
            "MatchesCompleted": 10,
            "CoreStats": { "ObjectivesCompleted": 1517 },
            "ZonesStats": {
                "ZoneCaptures": 4689,
                "TotalZoneOccupationTime": "PT21H6M9.7S",
                "ZoneScoringTicks": 28672
            },
            "PveStats": { "Kills": 46998, "GruntKills": 40461 }
        }))
        .unwrap();

        assert_eq!(record.core_stats.objectives_completed, 1517);
        let zones = record.zones_stats.as_ref().unwrap();
        assert_eq!(zones.zone_captures, 4689);
        assert_eq!(zones.total_zone_occupation_time, "PT21H6M9.7S");
        assert_eq!(record.pve_stats.as_ref().unwrap().grunt_kills, 40461);
        assert!(record.bomb_stats.is_none());
    }

    #[test]
    fn medal_metadata_resolves_indices() {
        // Shape captured from the live medals metadata document (camelCase, hyphenated sprite key).
        let metadata: MedalMetadata = serde_json::from_value(serde_json::json!({
            "difficulties": ["normal", "heroic", "legendary", "mythic"],
            "types": ["spree", "mode", "multikill", "proficiency", "skill", "style"],
            "sprites": {
                "small": { "path": "sm.png", "columns": 16, "size": 72 },
                "medium": { "path": "med.png", "columns": 16, "size": 128 },
                "extra-large": { "path": "xl.png", "columns": 16, "size": 256 }
            },
            "medals": [{
                "nameId": 622331684,
                "name": { "value": "Double Kill", "translations": { "de-DE": "Doppelter Abschuss" } },
                "description": { "value": "Kill 2 enemies", "translations": {} },
                "spriteIndex": 64,
                "sortingWeight": 100,
                "difficultyIndex": 1,
                "typeIndex": 2,
                "personalScore": 50
            }]
        }))
        .unwrap();

        let medal = metadata.medal(622331684).unwrap();
        assert_eq!(medal.name.value, "Double Kill");
        assert_eq!(metadata.difficulty_of(medal), Some("heroic"));
        assert_eq!(metadata.type_of(medal), Some("multikill"));
        assert_eq!(metadata.sprites.extra_large.size, 256);
    }

    #[test]
    fn deserializes_reward_track_and_operations_shapes() {
        let passes: PlayerOperationPasses = serde_json::from_value(serde_json::json!({
            "ActiveOperationRewardTrackPath": "RewardTracks/Operations/S05OpPassM01.json",
            "OperationRewardTracks": [{
                "RewardTrackPath": "RewardTracks/Operations/S05OpPassM01.json",
                "IsOwned": true,
                "CurrentProgress": { "Rank": 12, "PartialProgress": 300, "HasReachedMaxRank": false }
            }]
        }))
        .unwrap();
        assert_eq!(passes.operation_reward_tracks.len(), 1);
        assert!(passes.operation_reward_tracks[0].is_owned);
        assert_eq!(passes.operation_reward_tracks[0].current_progress.rank, 12);

        let track: CareerRewardTrack = serde_json::from_value(serde_json::json!({
            "TrackId": "careerRank1",
            "XpPerRank": 1000,
            "Ranks": [{
                "Rank": 1,
                "XpRequiredForRank": 0,
                "FreeRewards": {
                    "InventoryRewards": [{
                        "InventoryItemPath": "Inventory/Armor/Helmets/x.json",
                        "Amount": 1,
                        "Type": "ArmorHelmet"
                    }],
                    "CurrencyRewards": null
                },
                "PaidRewards": { "InventoryRewards": null, "CurrencyRewards": null }
            }]
        }))
        .unwrap();
        assert_eq!(track.track_id, "careerRank1");
        assert_eq!(track.ranks[0].free_rewards.inventory_rewards.len(), 1);
        assert!(track.ranks[0].paid_rewards.inventory_rewards.is_empty());
    }

    #[test]
    fn deserializes_discord_bot_response_contracts() {
        let privacy: MatchesPrivacy = serde_json::from_value(serde_json::json!({
            "MatchmadeGames": 1,
            "OtherGames": 2
        }))
        .unwrap();
        assert_eq!(privacy.matchmade_setting(), PrivacySetting::Show);
        assert_eq!(privacy.other_setting(), PrivacySetting::Hide);

        let service_record: ServiceRecord = serde_json::from_value(serde_json::json!({
            "MatchesCompleted": 3_000_000_000_i64,
            "CoreStats": {
                "Score": 4_000_000_000_i64,
                "DamageDealt": 5_000_000_000_i64
            }
        }))
        .unwrap();
        assert_eq!(service_record.matches_completed, 3_000_000_000);
        assert_eq!(service_record.core_stats.damage_dealt, 5_000_000_000);

        let empty_record: ServiceRecord = serde_json::from_value(serde_json::json!({
            "Subqueries": {
                "SeasonIds": null,
                "GameVariantCategories": null,
                "IsRanked": null,
                "PlaylistAssetIds": null,
                "GameplayInteractions": null
            },
            "CoreStats": { "Medals": [], "PersonalScores": [] }
        }))
        .unwrap();
        assert!(empty_record.subqueries.season_ids.is_empty());
        assert!(empty_record.subqueries.playlist_asset_ids.is_empty());

        let appearance: AppearanceCustomization = serde_json::from_value(serde_json::json!({
            "Status": "Success",
            "Appearance": {
                "LastModifiedDateUtc": { "ISO8601Date": "2026-01-01T00:00:00Z" },
                "ActionPosePath": "pose.json",
                "StancePath": null,
                "BackdropImagePath": "backdrop.json",
                "Emblem": { "EmblemPath": "emblem.json", "ConfigurationId": 42 },
                "ServiceTag": "117",
                "IntroEmotePath": null
            }
        }))
        .unwrap();
        assert_eq!(appearance.appearance.service_tag, "117");

        let emblem_mapping: EmblemMapping = serde_json::from_value(serde_json::json!({
            "104-001-reach-wrath-e-37d15c60": {
                "-1490538315": {
                    "emblemCmsPath": "images/emblems/wrath.png",
                    "nameplateCmsPath": "images/nameplates/wrath.png",
                    "textColor": "#000000"
                }
            }
        }))
        .unwrap();
        let emblem = emblem_mapping
            .get("104-001-reach-wrath-e-37d15c60", -1_490_538_315)
            .unwrap();
        assert_eq!(emblem.emblem_cms_path, "images/emblems/wrath.png");
        assert_eq!(emblem.text_color, "#000000");
        assert_eq!(
            emblem.emblem_url(),
            "https://gamecms-hacs.svc.halowaypoint.com/hi/Waypoint/file/images/emblems/wrath.png"
        );
        assert_eq!(
            emblem.nameplate_url(),
            "https://gamecms-hacs.svc.halowaypoint.com/hi/Waypoint/file/images/nameplates/wrath.png"
        );

        let equipped = EmblemConfiguration {
            emblem_path: "Inventory/Spartan/Emblems/104-001-reach-wrath-e-37d15c60.json"
                .to_string(),
            configuration_id: -1_490_538_315,
        };
        assert_eq!(
            emblem_mapping.resolve(&equipped).unwrap().emblem_cms_path,
            "images/emblems/wrath.png"
        );

        let metadata: EmblemMetadata = serde_json::from_value(serde_json::json!({
            "CommonData": {
                "Title": {
                    "status": "Ready",
                    "value": "The Gate",
                    "translations": { "de-DE": "Das Tor" }
                },
                "DisplayPath": {
                    "Media": {
                        "MediaUrl": {
                            "Path": "Progression/Inventory/Armor/Helmets/example.png"
                        }
                    }
                }
            }
        }))
        .unwrap();
        assert_eq!(metadata.common_data.title.value, "The Gate");
        assert_eq!(
            metadata.image_cms_path(),
            Some("Progression/Inventory/Armor/Helmets/example.png")
        );
        assert_eq!(metadata.image_id(), Some("example"));

        let applied_emblem: CustomizationEmblem = serde_json::from_value(serde_json::json!({
            "Path": "Inventory/Armor/Emblems/013-001-0fe4e1a4.json",
            "LocationId": -142056143,
            "ConfigurationId": 802727239
        }))
        .unwrap();
        assert_eq!(applied_emblem.emblem_id(), Some("013-001-0fe4e1a4"));

        let public: PlayerCustomizationCollection = serde_json::from_value(serde_json::json!({
            "PlayerCustomizations": [{
                "Id": "xuid(123)",
                "ResultCode": "Success",
                "Result": {
                    "Appearance": {
                        "LastModifiedDateUtc": null,
                        "ActionPosePath": "pose.json",
                        "StancePath": null,
                        "BackdropImagePath": "backdrop.json",
                        "Emblem": null,
                        "ServiceTag": "117",
                        "IntroEmotePath": null
                    },
                    "ArmorCores": {},
                    "SpartanBody": {
                        "BodyType": "Large",
                        "VoicePath": "Inventory/Spartan/Voices/voice.json",
                        "LeftArm": "None",
                        "RightArm": "None",
                        "LeftLeg": "None",
                        "RightLeg": "None",
                        "LastModifiedDateUtc": { "ISO8601Date": "2026-01-01T00:00:00Z" }
                    }
                }
            }]
        }))
        .unwrap();
        assert_eq!(public.player_customizations[0].result_code, "Success");
        assert_eq!(
            public.player_customizations[0]
                .result
                .spartan_body
                .as_ref()
                .unwrap()
                .body_type,
            "Large"
        );

        let armor_theme: ArmorTheme = serde_json::from_value(serde_json::json!({
            "ArmorFxPath": "",
            "ArmorFxPaths": null,
            "ChestAttachmentPath": "",
            "CoatingPath": "coating.json",
            "Emblems": [],
            "GlovePath": "",
            "HelmetAttachmentPath": "",
            "HelmetPath": "helmet.json",
            "HipAttachmentPath": "",
            "KneePadPath": "",
            "LeftShoulderPadPath": "",
            "MythicFxPath": "",
            "RightShoulderPadPath": "",
            "ThemePath": "theme.json",
            "VisorPath": "",
            "WristAttachmentPath": "",
            "FirstModifiedDateUtc": { "ISO8601Date": "2026-01-01T00:00:00Z" },
            "LastModifiedDateUtc": { "ISO8601Date": "2026-01-01T00:00:00Z" },
            "IsDefault": true,
            "IsEquipped": true
        }))
        .unwrap();
        assert!(armor_theme.armor_fx_paths.is_empty());

        let bans: BanSummary = serde_json::from_value(serde_json::json!({
            "Results": [{
                "Id": "xuid(123)",
                "ResultCode": 0,
                "Result": { "BansInEffect": [{
                    "Type": 1,
                    "Scope": 1,
                    "EnforceUntilUtc": { "ISO8601Date": "2026-08-01T00:00:00Z" },
                    "BanMessagePath": "Banning/example.json"
                }] }
            }]
        }))
        .unwrap();
        assert_eq!(bans.results[0].result.bans_in_effect.len(), 1);

        let message: BanMessage = serde_json::from_value(serde_json::json!({
            "Title": "HI: Admin Matchmaking Ban Cheating",
            "DisplayMessage": {
                "status": "Ready",
                "value": "Suspended until {0}",
                "translations": { "de-DE": "Gesperrt bis {0}" }
            }
        }))
        .unwrap();
        assert_eq!(message.display_message.translations.len(), 1);

        let scoreboard: MatchStats = serde_json::from_value(serde_json::json!({
            "MatchId": "match",
            "MatchInfo": {
                "StartTime": "2026-01-01T00:00:00Z",
                "EndTime": "2026-01-01T00:10:00Z",
                "Duration": "PT10M",
                "GameVariantCategory": 6,
                "MapVariant": { "AssetKind": 2, "AssetId": "map", "VersionId": "v1" },
                "UgcGameVariant": { "AssetKind": 6, "AssetId": "mode", "VersionId": "v2" },
                "Playlist": { "AssetId": "playlist" }
            },
            "Teams": [],
            "Players": [{
                "PlayerId": "xuid(123)",
                "PlayerType": 1,
                "LastTeamId": 0,
                "Outcome": 2,
                "Rank": 1,
                "PlayerTeamStats": [{
                    "TeamId": 0,
                    "Stats": { "CoreStats": {
                        "PersonalScore": 2500, "Kills": 20, "Deaths": 10, "Assists": 5
                    }}
                }]
            }]
        }))
        .unwrap();
        assert_eq!(scoreboard.players[0].outcome, MatchOutcome::Win);
        assert_eq!(scoreboard.players[0].team_stats[0].stats.core.kills, 20);
    }

    #[test]
    fn falls_back_to_a_positive_configuration_when_equipped_one_is_unmapped() {
        let configuration = EmblemConfiguration {
            emblem_path: "Inventory/Spartan/Emblems/3806589-SpartanEmblem.json".to_string(),
            configuration_id: -1_766_636_888,
        };
        let metadata: EmblemMetadata = serde_json::from_value(serde_json::json!({
            "CommonData": {
                "Title": { "status": "Ready", "value": "Women's History Month", "translations": {} }
            },
            "AvailableConfigurations": [
                { "ConfigurationId": -1_766_636_888 },
                { "ConfigurationId": 651_339_664 }
            ]
        }))
        .unwrap();
        assert_eq!(
            metadata.first_positive_configuration_id(),
            Some(651_339_664)
        );

        let mapping: EmblemMapping = serde_json::from_value(serde_json::json!({
            "3806589-SpartanEmblem": {
                "651339664": {
                    "emblemCmsPath": "images/emblems/whm.png",
                    "nameplateCmsPath": "images/nameplates/whm.png",
                    "textColor": "#FFFFFF"
                }
            }
        }))
        .unwrap();

        // The equipped configuration (-1_766_636_888, a "Test" palette) is not mapped.
        assert!(mapping.resolve(&configuration).is_none());
        // Falling back finds the emblem's other, mapped configuration instead.
        assert_eq!(
            mapping
                .resolve_with_fallback(&configuration, &metadata)
                .unwrap()
                .emblem_cms_path,
            "images/emblems/whm.png"
        );
    }

    #[test]
    fn fallback_returns_none_when_emblem_has_no_positive_configuration() {
        // The real-world 3806589-SpartanEmblem case: readable CMS JSON, but no positive
        // configuration exists at all, so no fallback image can be found either.
        let configuration = EmblemConfiguration {
            emblem_path: "Inventory/Spartan/Emblems/3806589-SpartanEmblem.json".to_string(),
            configuration_id: -1_766_636_888,
        };
        let metadata: EmblemMetadata = serde_json::from_value(serde_json::json!({
            "CommonData": {
                "Title": { "status": "Ready", "value": "Women's History Month", "translations": {} }
            },
            "AvailableConfigurations": [{ "ConfigurationId": -1_766_636_888 }]
        }))
        .unwrap();
        let mapping = EmblemMapping {
            emblems: Default::default(),
        };
        assert!(
            mapping
                .resolve_with_fallback(&configuration, &metadata)
                .is_none()
        );
    }

    #[test]
    fn falls_back_to_a_different_mapped_color_variant_of_the_same_emblem() {
        // The player's equipped color variant (a positive, "real" configuration — not a
        // negative "Test" palette) simply isn't in Halo's curated mapping table, but a
        // different color variant of the same emblem is.
        let configuration = EmblemConfiguration {
            emblem_path: "Inventory/Spartan/Emblems/104-001-samurai-thega-d83dae85.json"
                .to_string(),
            configuration_id: 999_999_999,
        };
        let metadata: EmblemMetadata = serde_json::from_value(serde_json::json!({
            "CommonData": {
                "Title": { "status": "Ready", "value": "Samurai", "translations": {} }
            },
            "AvailableConfigurations": [
                { "ConfigurationId": 999_999_999 },
                { "ConfigurationId": 1_198_260_756 }
            ]
        }))
        .unwrap();
        // The mapping only has the *other* positive configuration — not the equipped one, and
        // not the metadata's first positive configuration ID either.
        let mapping: EmblemMapping = serde_json::from_value(serde_json::json!({
            "104-001-samurai-thega-d83dae85": {
                "-42": {
                    "emblemCmsPath": "images/emblems/samurai-negative.png",
                    "nameplateCmsPath": "images/nameplates/samurai-negative.png",
                    "textColor": "#000000"
                }
            }
        }))
        .unwrap();

        assert!(mapping.resolve(&configuration).is_none());
        assert_eq!(
            mapping
                .resolve_with_fallback(&configuration, &metadata)
                .unwrap()
                .emblem_cms_path,
            "images/emblems/samurai-negative.png"
        );
    }

    #[test]
    fn first_positive_configuration_id_returns_none_without_one() {
        let metadata: EmblemMetadata = serde_json::from_value(serde_json::json!({
            "CommonData": { "Title": { "status": "Ready", "value": "Test", "translations": {} } },
            "AvailableConfigurations": [{ "ConfigurationId": -1_766_636_888 }]
        }))
        .unwrap();
        assert_eq!(metadata.first_positive_configuration_id(), None);
    }
}
