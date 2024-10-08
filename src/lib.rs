use exports::provider::{Dict, EdgeeRequest, Guest, Payload};
use ga_payload::GaPayload;
use std::collections::HashMap;

mod ga_payload;

wit_bindgen::generate!({world: "data-collection"});
export!(GaComponent);

struct GaComponent;

impl Guest for GaComponent {
    fn page(edgee_payload: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        let mut ga_payload = GaPayload::new(&edgee_payload, cred_map, String::from("page_view"))
            .map_err(|e| e.to_string())?;
        ga_payload.document_location = edgee_payload.page.url.clone();
        ga_payload.document_title = edgee_payload.page.title.clone();
        ga_payload.document_referrer = Some(edgee_payload.page.referrer.clone());

        let mut event_parameter_string = HashMap::new();
        let mut event_parameter_number = HashMap::new();

        if !edgee_payload.page.name.is_empty() {
            event_parameter_string
                .insert(String::from("page_name"), edgee_payload.page.name.clone());
        }

        if !edgee_payload.page.category.is_empty() {
            event_parameter_string.insert(
                String::from("page_category"),
                edgee_payload.page.category.clone(),
            );
        }

        if !edgee_payload.page.keywords.is_empty() {
            event_parameter_string.insert(
                String::from("page_keywords"),
                edgee_payload.page.keywords.join(","),
            );
        }

        if !edgee_payload.page.search.is_empty() {
            event_parameter_string.insert(
                String::from("page_search"),
                edgee_payload.page.search.clone(),
            );
        }

        for (key, value) in edgee_payload.page.properties.iter() {
            let key = key.replace(" ", "_");
            if let Some(value) = value.parse::<f64>().ok() {
                event_parameter_number.insert(key, value);
            } else {
                event_parameter_string.insert(key, value.to_string().trim_matches('"').to_string());
            }
        }

        if event_parameter_string.len() > 0 {
            ga_payload.event_parameter_string = Some(event_parameter_string);
        }
        if event_parameter_number.len() > 0 {
            ga_payload.event_parameter_number = Some(event_parameter_number);
        }

        Ok(build_edgee_request(ga_payload).map_err(|e| e.to_string())?)
    }

    fn track(edgee_payload: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if edgee_payload.track.name.is_empty() {
            return Err("Track is not set".to_string());
        }

        let mut ga_payload = GaPayload::new(
            &edgee_payload,
            cred_map,
            String::from(edgee_payload.track.name.clone()),
        )
        .map_err(|e| e.to_string())?;

        let mut event_parameter_string = HashMap::new();
        let mut event_parameter_number = HashMap::new();

        for (key, value) in edgee_payload.track.properties.iter() {
            let key = key.replace(" ", "_");
            if let Some(value) = value.parse::<f64>().ok() {
                event_parameter_number.insert(key, value);
            } else {
                event_parameter_string.insert(key, value.to_string().trim_matches('"').to_string());
            }
        }

        if event_parameter_string.len() > 0 {
            ga_payload.event_parameter_string = Some(event_parameter_string);
        }
        if event_parameter_number.len() > 0 {
            ga_payload.event_parameter_number = Some(event_parameter_number);
        }

        Ok(build_edgee_request(ga_payload).map_err(|e| e.to_string())?)
    }

    fn identify(edgee_payload: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if edgee_payload.identify.user_id.is_empty()
            && edgee_payload.identify.anonymous_id.is_empty()
        {
            return Err("userId or anonymousId is not set".to_string());
        }

        let ga_payload = GaPayload::new(&edgee_payload, cred_map, String::from("identify"))
            .map_err(|e| e.to_string())?;

        Ok(build_edgee_request(ga_payload).map_err(|e| e.to_string())?)
    }
}

fn build_edgee_request(ga_payload: GaPayload) -> anyhow::Result<EdgeeRequest> {
    let mut headers = vec![];
    headers.push((String::from("content-length"), String::from("0")));

    let querystring = serde_qs::to_string(&ga_payload)?;

    let querystring = cleanup_querystring(&querystring)?;

    Ok(EdgeeRequest {
        method: exports::provider::HttpMethod::Post,
        url: format!("https://www.google-analytics.com/g/collect?{}", querystring),
        headers,
        body: String::new(),
    })
}

// change every parameters like ep[page_type]=checkout to ep.page_type=checkout. check only the ep, epn, up, upn parameters
fn cleanup_querystring(ga4_qs: &str) -> anyhow::Result<String> {
    // regex, replace ep[page_type]=checkout to ep.page_type=checkout
    let re = regex::Regex::new(r#"ep\[(\w+)\]"#).unwrap();
    let ga4_qs = re.replace_all(&ga4_qs, "ep.$1");

    // regex, replace epn[page_number]=123 to epn.page_number=123
    let re = regex::Regex::new(r#"epn\[(\w+)\]"#).unwrap();
    let ga4_qs = re.replace_all(&ga4_qs, "epn.$1");

    // regex, replace up[user_type]=premium to up.user_type=premium
    let re = regex::Regex::new(r#"up\[(\w+)\]"#).unwrap();
    let ga4_qs = re.replace_all(&ga4_qs, "up.$1");

    // regex, replace upn[lifetime_value]=45.50 to upn.lifetime_value=45.50
    let re = regex::Regex::new(r#"upn\[(\w+)\]"#).unwrap();
    let ga4_qs = re.replace_all(&ga4_qs, "upn.$1");

    Ok(ga4_qs.parse()?)
}
