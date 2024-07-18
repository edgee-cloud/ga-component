use std::collections::HashMap;

use exports::provider::{Dict, EdgeeRequest, Guest, Payload};
use rand::RngCore;
use serde::Serialize;

mod string_ext;
use string_ext::StringExt;

wit_bindgen::generate!({world: "data-collection"});
export!(GaComponent);

struct GaComponent;

impl GaComponent {
    fn build_headers(p: &Payload) -> Vec<(String, String)> {
        let mut headers = vec![];
        headers.push((
            String::from("content-type"),
            String::from("application/json"),
        ));
        headers.push((
            String::from("user-agent"),
            String::from(&p.client.user_agent),
        ));
        headers.push((String::from("x-forwarded-for"), String::from(&p.client.ip)));
        return headers;
    }
}

impl Guest for GaComponent {
    fn page(p: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        let cred: HashMap<String, String> = cred_map
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        let mut m = GaMeasurement::new(p.clone(), cred.clone(), String::from("page_view"));
        m.document_location = p.page.url.clone();
        m.document_title = p.page.title.clone();
        m.document_referrer = p.page.referrer.clone();

        let mut event_parameter_string = HashMap::new();
        let mut event_parameter_number = HashMap::new();

        if !p.page.name.is_empty() {
            event_parameter_string.insert(String::from("page_name"), p.page.name.clone());
        }

        if !p.page.category.is_empty() {
            event_parameter_string.insert(String::from("page_category"), p.page.category.clone());
        }

        if !p.page.keywords.is_empty() {
            event_parameter_string.insert(String::from("page_keywords"), p.page.keywords.join(","));
        }

        if !p.page.search.is_empty() {
            event_parameter_string.insert(String::from("page_search"), p.page.search.clone());
        }

        for (key, value) in p.page.properties.iter() {
            let key = key.replace(" ", "_");
            if let Some(value) = value.parse::<f64>().ok() {
                event_parameter_number.insert(key, value);
            } else {
                event_parameter_string.insert(key, value.to_string());
            }
        }

        m.event_parameter_string = event_parameter_string;
        m.event_parameter_number = event_parameter_number;

        let querystring = serde_qs::to_string(&m)
            .map_err(|e| e.to_string())
            .unwrap_or_default();

        let querystring = cleanup_querystring(&querystring).ok().unwrap_or_default();

        Ok(EdgeeRequest {
            method: exports::provider::HttpMethod::Post,
            url: format!("https://www.google-analytics.com/g/collect?{}", querystring),
            headers: GaComponent::build_headers(&p),
            body: String::new(),
        })
    }

    fn track(p: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        let cred: HashMap<String, String> = cred_map
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        if p.track.name.is_empty() {
            return Err("Track is not set".to_string());
        }

        let mut m = GaMeasurement::new(p.clone(), cred.clone(), String::from(p.track.name.clone()));

        let mut event_parameter_string = HashMap::new();
        let mut event_parameter_number = HashMap::new();

        for (key, value) in p.track.properties.iter() {
            let key = key.replace(" ", "_");
            if let Some(value) = value.parse::<f64>().ok() {
                event_parameter_number.insert(key, value);
            } else {
                event_parameter_string.insert(key, value.to_string());
            }
        }

        m.event_parameter_string = event_parameter_string;
        m.event_parameter_number = event_parameter_number;

        let querystring = serde_qs::to_string(&m)
            .map_err(|e| e.to_string())
            .unwrap_or_default();

        let querystring = cleanup_querystring(&querystring).ok().unwrap_or_default();

        Ok(EdgeeRequest {
            method: exports::provider::HttpMethod::Post,
            url: format!("https://www.google-analytics.com/g/collect?{}", querystring),
            headers: GaComponent::build_headers(&p),
            body: String::new(),
        })
    }

    fn identify(p: Payload, cred_map: Dict) -> Result<EdgeeRequest, _rt::String> {
        let cred: HashMap<String, String> = cred_map
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        let m = GaMeasurement::new(p.clone(), cred.clone(), String::from("identify"));
        let querystring = serde_qs::to_string(&m)
            .map_err(|e| e.to_string())
            .unwrap_or_default();

        let querystring = cleanup_querystring(&querystring).ok().unwrap_or_default();

        Ok(EdgeeRequest {
            method: exports::provider::HttpMethod::Post,
            url: format!("https://www.google-analytics.com/g/collect?{}", querystring),
            headers: GaComponent::build_headers(&p),
            body: String::new(),
        })
    }
}

#[derive(Debug, Default, Serialize)]
struct GaMeasurement {
    #[serde(rename = "v")]
    protocol_version: String,
    #[serde(rename = "tid")]
    tracking_id: String,
    #[serde(rename = "gtm", skip_serializing_if = "String::is_empty")]
    gtm_hash_info: String,
    #[serde(rename = "_p")]
    random_page_load_hash: String,

    #[serde(rename = "sr", skip_serializing_if = "String::is_empty")]
    screen_resolution: String,
    #[serde(rename = "ul")]
    user_language: String,
    #[serde(rename = "dh", skip_serializing_if = "String::is_empty")]
    document_hostname: String,
    #[serde(rename = "cid")]
    client_id: String,
    #[serde(rename = "_s")]
    hit_counter: String,
    #[serde(rename = "richsstsse", skip_serializing_if = "String::is_empty")]
    richsstsse: String,

    #[serde(rename = "id", skip_serializing_if = "String::is_empty")]
    item_id: String,
    #[serde(rename = "br", skip_serializing_if = "String::is_empty")]
    item_brand: String,
    #[serde(rename = "ca", skip_serializing_if = "String::is_empty")]
    item_category_hierarchy1: String,
    #[serde(rename = "ca2", skip_serializing_if = "String::is_empty")]
    item_category_hierarchy2: String,
    #[serde(rename = "ca3", skip_serializing_if = "String::is_empty")]
    item_category_hierarchy3: String,
    #[serde(rename = "ca4", skip_serializing_if = "String::is_empty")]
    item_category_hierarchy4: String,
    #[serde(rename = "ca5", skip_serializing_if = "String::is_empty")]
    item_category_hierarchy5: String,
    #[serde(rename = "pr", skip_serializing_if = "String::is_empty")]
    item_price: String,
    #[serde(rename = "qt", skip_serializing_if = "String::is_empty")]
    item_quantity: String,
    #[serde(rename = "va", skip_serializing_if = "String::is_empty")]
    item_variant: String,
    #[serde(rename = "cp", skip_serializing_if = "String::is_empty")]
    item_coupon: String,
    #[serde(rename = "ds", skip_serializing_if = "String::is_empty")]
    item_discount: String,
    #[serde(rename = "ln", skip_serializing_if = "String::is_empty")]
    item_list_name: String,
    #[serde(rename = "li", skip_serializing_if = "String::is_empty")]
    item_list_id: String,
    #[serde(rename = "lp", skip_serializing_if = "String::is_empty")]
    item_list_position: String,
    #[serde(rename = "af", skip_serializing_if = "String::is_empty")]
    item_affiliation: String,

    #[serde(rename = "uaa", skip_serializing_if = "String::is_empty")]
    user_agent_architecture: String,
    #[serde(rename = "uab", skip_serializing_if = "String::is_empty")]
    user_agent_bitness: String,
    #[serde(rename = "uafvl", skip_serializing_if = "String::is_empty")]
    user_agent_full_version_list: String,
    #[serde(rename = "uamb", skip_serializing_if = "String::is_empty")]
    user_agent_mobile: String,
    #[serde(rename = "uam", skip_serializing_if = "String::is_empty")]
    user_agent_model: String,
    #[serde(rename = "uap", skip_serializing_if = "String::is_empty")]
    user_agent_platform: String,
    #[serde(rename = "uapv", skip_serializing_if = "String::is_empty")]
    user_agent_platform_version: String,
    #[serde(rename = "uaw", skip_serializing_if = "String::is_empty")]
    user_agent_wow64: String,

    #[serde(rename = "dl")]
    document_location: String,
    #[serde(rename = "dt")]
    document_title: String,
    #[serde(rename = "dr", skip_serializing_if = "String::is_empty")]
    document_referrer: String,
    #[serde(rename = "_z", skip_serializing_if = "String::is_empty")]
    z: String,
    #[serde(rename = "_eu", skip_serializing_if = "String::is_empty")]
    event_usage: String,
    #[serde(rename = "edid", skip_serializing_if = "String::is_empty")]
    event_debug_id: String,
    #[serde(rename = "_dbg", skip_serializing_if = "String::is_empty")]
    is_debug: String,
    #[serde(rename = "ir", skip_serializing_if = "String::is_empty")]
    ignore_referrer: String,
    #[serde(rename = "tt", skip_serializing_if = "String::is_empty")]
    traffic_type: String,
    #[serde(rename = "gcs", skip_serializing_if = "String::is_empty")]
    google_consent_status: String,
    #[serde(rename = "gcu", skip_serializing_if = "String::is_empty")]
    google_consent_update: String,
    #[serde(rename = "gcut", skip_serializing_if = "String::is_empty")]
    google_consent_update_type: String,
    #[serde(rename = "gcd", skip_serializing_if = "String::is_empty")]
    google_consent_default: String,
    #[serde(rename = "_glv", skip_serializing_if = "String::is_empty")]
    is_google_linker_valid: String,

    #[serde(rename = "cm", skip_serializing_if = "String::is_empty")]
    campaign_medium: String,
    #[serde(rename = "cs", skip_serializing_if = "String::is_empty")]
    campaign_source: String,
    #[serde(rename = "cn", skip_serializing_if = "String::is_empty")]
    campaign_name: String,
    #[serde(rename = "cc", skip_serializing_if = "String::is_empty")]
    campaign_content: String,
    #[serde(rename = "ck", skip_serializing_if = "String::is_empty")]
    campaign_term: String,
    #[serde(rename = "ccf", skip_serializing_if = "String::is_empty")]
    campaign_creative_format: String,
    #[serde(rename = "cmt", skip_serializing_if = "String::is_empty")]
    campaign_marketing_tactic: String,
    #[serde(rename = "_rnd", skip_serializing_if = "String::is_empty")]
    gclid_deduper: String,

    #[serde(rename = "en", skip_serializing_if = "String::is_empty")]
    event_name: String,
    #[serde(rename = "_et", skip_serializing_if = "String::is_empty")]
    engagement_time: String,
    #[serde(rename = "ep", skip_serializing_if = "HashMap::is_empty")]
    event_parameter_string: HashMap<String, String>,
    #[serde(rename = "epn", skip_serializing_if = "HashMap::is_empty")]
    event_parameter_number: HashMap<String, f64>,
    #[serde(rename = "_c", skip_serializing_if = "String::is_empty")]
    is_conversion: String,
    #[serde(rename = "_ee", skip_serializing_if = "String::is_empty")]
    external_event: String,

    #[serde(rename = "uid", skip_serializing_if = "String::is_empty")]
    user_id: String,
    #[serde(rename = "_fid", skip_serializing_if = "String::is_empty")]
    firebase_id: String,
    #[serde(rename = "sid", skip_serializing_if = "String::is_empty")]
    session_id: String,
    #[serde(rename = "sct", skip_serializing_if = "String::is_empty")]
    session_count: String,
    #[serde(rename = "seg", skip_serializing_if = "String::is_empty")]
    session_engagement: String,
    #[serde(rename = "up", skip_serializing_if = "HashMap::is_empty")]
    user_property_string: HashMap<String, String>,
    #[serde(rename = "upn", skip_serializing_if = "HashMap::is_empty")]
    user_property_number: HashMap<String, f64>,
    #[serde(rename = "_fv", skip_serializing_if = "String::is_empty")]
    first_visit: String,
    #[serde(rename = "_ss", skip_serializing_if = "String::is_empty")]
    session_start: String,
    #[serde(rename = "_fplc", skip_serializing_if = "String::is_empty")]
    first_party_linker_cookie: String,
    #[serde(rename = "_nsi", skip_serializing_if = "String::is_empty")]
    new_session_id: String,
    #[serde(rename = "_gdid", skip_serializing_if = "String::is_empty")]
    google_developer_id: String,
    #[serde(rename = "_uc", skip_serializing_if = "String::is_empty")]
    user_country: String,
}

impl GaMeasurement {
    fn new(p: Payload, cred: HashMap<String, String>, event_name: String) -> Self {
        let mut data = Self::default();
        data.protocol_version = String::from("2");
        data.event_name = event_name;
        data.random_page_load_hash = random_page_load_hash();
        data.external_event = String::from("1");
        data.tracking_id = cred
            .get("ga_measurement_id")
            .map(String::from)
            .unwrap_or_default();

        let first_seen = p.session.first_seen.clone();
        let ga_user_id = uuid_hash(&p.identify.user_id).unwrap_or_default();
        data.client_id = format!("{}.{}", ga_user_id, first_seen);
        data.hit_counter = String::from("1");

        data.user_language = p.client.locale.or("en").to_string();
        data.user_agent_full_version_list = p.client.user_agent_full_version_list.clone();
        data.user_agent_mobile = p.client.user_agent_mobile.clone();
        data.user_agent_platform = p.client.os_name.clone();
        data.user_agent_platform_version = p.client.os_version.clone();
        data.user_agent_architecture = p.client.user_agent_architecture.clone();
        data.user_agent_bitness = p.client.user_agent_bitness.clone();
        data.user_agent_model = p.client.user_agent_model.clone();

        if p.client.screen_width != 0 && !p.client.screen_height != 0 {
            data.screen_resolution =
                format!("{}x{}", p.client.screen_width, p.client.screen_height);
        }

        let mut user_property_string = HashMap::new();
        let mut user_property_number = HashMap::new();

        if !p.identify.anonymous_id.is_empty() {
            data.user_id = p.identify.anonymous_id.clone();
        }

        if !p.identify.user_id.is_empty() {
            data.user_id = p.identify.user_id.clone();
            if !p.identify.anonymous_id.is_empty() {
                user_property_string.insert(
                    String::from("anonymous_id"),
                    p.identify.anonymous_id.clone(),
                );
            }
        }

        for (key, value) in p.identify.properties.iter() {
            let key = key.replace(" ", "_");
            if let Some(value) = value.parse::<f64>().ok() {
                user_property_number.insert(key, value);
            } else {
                user_property_string.insert(key, value.to_string());
            }
        }
        data.user_property_string = user_property_string;
        data.user_property_number = user_property_number;

        data.user_country = p.client.country_code.clone();

        data.campaign_medium = p.campaign.medium.clone();
        data.campaign_source = p.campaign.source.clone();
        data.campaign_name = p.campaign.name.clone();
        data.campaign_content = p.campaign.content.clone();
        data.campaign_term = p.campaign.term.clone();

        data.session_id = p.session.session_id.clone();
        data.session_count = p.session.session_count.to_string();

        if p.session.first_seen == p.session.last_seen {
            data.first_visit = String::from("1");
            data.new_session_id = String::from("1");
        }

        if p.session.session_start {
            data.session_start = String::from("1");
            data.session_engagement = String::from("0");
        } else {
            data.session_engagement = String::from("1");
        }

        return data;
    }
}

fn cleanup_querystring(querystring: &str) -> Result<String, ()> {
    let re = regex::Regex::new(r#"ep\[(\w+)\]"#).map_err(|_| ())?;
    let querystring = re.replace_all(&querystring, "ep.$1");

    let re = regex::Regex::new(r#"epn\[(\w+)\]"#).map_err(|_| ())?;
    let querystring = re.replace_all(&querystring, "epn.$1");

    let re = regex::Regex::new(r#"up\[(\w+)\]"#).map_err(|_| ())?;
    let querystring = re.replace_all(&querystring, "up.$1");

    let re = regex::Regex::new(r#"upn\[(\w+)\]"#).map_err(|_| ())?;
    let querystring = re.replace_all(&querystring, "upn.$1");

    Ok(querystring.to_string())
}

fn random_page_load_hash() -> String {
    let mut rng = rand::thread_rng();
    format!("{:x}", rng.next_u32())
}

fn uuid_hash(input: &str) -> anyhow::Result<String> {
    let uuid = uuid::Uuid::parse_str(input)?;
    let modulo = 10u128.pow(10);
    let output = uuid.as_u128() % modulo;
    Ok(output.to_string())
}
