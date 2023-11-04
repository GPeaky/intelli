include!(concat!(env!("OUT_DIR"), "/protos.participants.rs"));

use super::ToProtoMessage;
use crate::dtos::PacketParticipantsData as BPacketParticipantsData;
use std::ffi::CStr;

impl ToProtoMessage for BPacketParticipantsData {
    type ProtoType = PacketParticipantsData;

    fn to_proto(&self) -> Self::ProtoType {
        PacketParticipantsData {
            m_num_active_cars: self.m_numActiveCars as u32,
            m_participants: self
                .m_participants
                .into_iter()
                .map(|value| {
                    let c_str = CStr::from_bytes_until_nul(&value.m_name).unwrap();

                    ParticipantData {
                        m_ai_controlled: value.m_aiControlled as u32,
                        m_driver_id: value.m_driverId as u32,
                        m_network_id: value.m_networkId as u32,
                        m_team_id: value.m_teamId as u32,
                        m_my_team: value.m_myTeam as u32,
                        m_race_number: value.m_raceNumber as u32,
                        m_nationality: value.m_nationality as u32,
                        m_name: c_str.to_str().unwrap().to_string(),
                        m_your_telemetry: value.m_yourTelemetry as u32,
                        m_show_online_names: value.m_showOnlineNames as u32,
                        m_platform: value.m_platform as u32,
                    }
                })
                .collect(),
        }
    }
}
