use std::{sync::Arc, time::Duration};

use webrtc::{api::APIBuilder, ice_transport::ice_server::RTCIceServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = APIBuilder::new().build();

    let config = webrtc::peer_connection::configuration::RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let offerer = Arc::new(api.new_peer_connection(config.clone()).await.unwrap());
    let answerer = Arc::new(api.new_peer_connection(config).await.unwrap());

    let offer_dc = offerer.create_data_channel("audio", None).await?;

    let odc = offer_dc.clone();
    offer_dc.on_open(Box::new(move || {
        println!("offer channel open");
        Box::pin(async move {
            odc.send_text("Hello from Offerer!").await.unwrap();
        })
    }));

    offer_dc.on_message(Box::new(move |msg| {
        println!(
            "echoing answerer: {}",
            String::from_utf8(msg.data.to_vec()).unwrap()
        );
        Box::pin(async {})
    }));

    answerer.on_data_channel(Box::new(|dc| {
        println!("Answerer: got data channel '{}'", dc.label());
        Box::pin(async move {
            dc.on_open(Box::new(move || {
                println!("Answerer data channel open");
                Box::pin(async {})
            }));

            dc.clone().on_message(Box::new(move |msg| {
                println!(
                    "Answerer got message: {}",
                    String::from_utf8_lossy(&msg.data)
                );

                let dc = dc.clone();
                Box::pin(async move {
                    dc.send(&msg.data).await.unwrap();
                })
            }));
        })
    }));

    offerer.on_peer_connection_state_change(Box::new(move |state| {
        println!("Offerer connection state: {:?}", state);
        Box::pin(async {})
    }));

    answerer.on_peer_connection_state_change(Box::new(move |state| {
        println!("Answerer connection state: {:?}", state);
        Box::pin(async {})
    }));

    offerer.on_ice_gathering_state_change(Box::new(move |state| {
        println!("Offerer ICE gathering state: {:?}", state);
        Box::pin(async {})
    }));
    answerer.on_ice_gathering_state_change(Box::new(move |state| {
        println!("Answerer ICE gathering state: {:?}", state);
        Box::pin(async {})
    }));

    let ans = answerer.clone();
    offerer.on_ice_candidate(Box::new(move |candidate| {
        let ans = ans.clone();
        Box::pin(async move {
            if let Some(candidate) = candidate {
                // Forward to answerer
                let candidate_init = candidate.to_json().unwrap();
                ans.add_ice_candidate(candidate_init).await.unwrap();
            }
        })
    }));

    let off = offerer.clone();
    answerer.on_ice_candidate(Box::new(move |candidate| {
        let off = off.clone();
        Box::pin(async move {
            if let Some(candidate) = candidate {
                let candidate_init = candidate.to_json().unwrap();
                off.add_ice_candidate(candidate_init).await.unwrap();
            }
        })
    }));

    // Do the SDP trade!
    let offer = offerer.create_offer(None).await?;
    offerer.set_local_description(offer.clone()).await?;

    answerer.set_remote_description(offer.clone()).await?;
    let answer = answerer.create_answer(None).await?;

    answerer.set_local_description(answer.clone()).await?;
    offerer.set_remote_description(answer.clone()).await?;

    tokio::time::sleep(Duration::from_secs(60 * 60 * 24)).await;

    Ok(())
}
