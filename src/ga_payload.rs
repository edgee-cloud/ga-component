use anyhow::anyhow;
use chrono::Utc;
use num_bigint::{BigInt, ToBigInt};
use num_traits::Num;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use std::collections::HashMap;

use crate::exports::edgee::components::data_collection::{Consent, Dict, Event};

/// from https://www.thyngster.com/ga4-measurement-protocol-cheatsheet/
#[derive(Serialize, Debug, Default)]
pub(crate) struct GaPayload {
    /// Defines que current protocol version being used. ex : 2
    #[serde(rename = "v")]
    protocol_version: String,
    /// Current Stream ID / Measurement ID. ex: G-4SNFB1GX9P
    #[serde(rename = "tid")]
    tracking_id: String,
    /// If the current hit is coming was generated from GTM, it will contain a hash of current GTM/GTAG config. ex: 45je34c0
    #[serde(rename = "gtm", skip_serializing_if = "Option::is_none")]
    gmt_hash_info: Option<String>,
    /// Is a random hash generated on the page load. Ex: 456193680 @javascript: Math.floor(Math.random() * (2147483647 - 0 + 1) + 0)
    #[serde(rename = "_p")]
    random_page_load_hash: String,

    /// Browser screen resolution in format width x height. Ex: 1512x982 @javascript: (window.screen ? window.screen.width : 0) + "x" + (window.screen ? window.screen.height : 0)
    #[serde(rename = "sr", skip_serializing_if = "Option::is_none")]
    screen_resolution: Option<String>,
    /// Browser active locale. ex: fr-fr @javascript: (navigator.language || navigator.userLanguage || navigator.browserLanguage || navigator.systemLanguage || "en").toLowerCase()
    #[serde(rename = "ul")]
    user_language: String,
    /// Current Document Hostname. ex: www.edgee.dev
    #[serde(rename = "dh", skip_serializing_if = "Option::is_none")]
    document_hostname: Option<String>,
    /// Google Analytics Client Id. ex: 944266522.1681654733 @javascript: ('; ' + document.cookie).split('; _ga=').pop().split(';').shift().match(/GA1\.[0-9]{1}\.(.+)/)[1]
    #[serde(rename = "cid")]
    client_id: String,
    /// Current hits counter for the current page load. ex: 1
    #[serde(rename = "_s")]
    hit_counter: String,
    /// This is supposed to be to enrich the GA4 hits to send data to SGTM, at this point is always set as an empty value...
    #[serde(rename = "richsstsse", skip_serializing_if = "Option::is_none")]
    richsstsse: Option<String>,

    // Client hints
    /// User Agent Architecture. ex: arm
    #[serde(rename = "uaa", skip_serializing_if = "Option::is_none")]
    user_agent_architecture: Option<String>,
    /// The "bitness" of the user-agent's underlying CPU architecture. This is the size in bits of an integer or memory address—typically 64 or 32 bits. ex: 64
    #[serde(rename = "uab", skip_serializing_if = "Option::is_none")]
    user_agent_bitness: Option<String>,
    /// The brand and full version information for each brand associated with the browser, in a comma-separated list. ex: Chromium;112.0.5615.49|Google%20Chrome;112.0.5615.49|Not%3AA-Brand;99.0.0.0
    #[serde(rename = "uafvl", skip_serializing_if = "Option::is_none")]
    user_agent_full_version_list: Option<String>,
    /// Indicates whether the browser is on a mobile device. ex: 0
    #[serde(rename = "uamb", skip_serializing_if = "Option::is_none")]
    user_agent_mobile: Option<String>,
    /// The device model on which the browser is running. Will likely be empty for desktop browsers. ex: Nexus 6
    #[serde(rename = "uam", skip_serializing_if = "Option::is_none")]
    user_agent_model: Option<String>,
    /// The platform or operating system on which the user agent is running. Ex: macOS
    #[serde(rename = "uap", skip_serializing_if = "Option::is_none")]
    user_agent_platform: Option<String>,
    /// The version of the operating system on which the user agent is running. Ex: 12.2.1
    #[serde(rename = "uapv", skip_serializing_if = "Option::is_none")]
    user_agent_platform_version: Option<String>,
    /// Whatever Windows On Windows 64 Bit is supported. Used by "WoW64-ness" sites. ( running 32bits app on 64bits windows). Ex: 1
    #[serde(rename = "uaw", skip_serializing_if = "Option::is_none")]
    user_agent_wow64: Option<String>,

    // Shared
    /// Actual page's Pathname. It does not include the hostname, queryString or fragment. ex: /bonjour @javascript: document.location.pathname
    #[serde(rename = "dl")]
    pub document_location: String,
    /// Actual page's Title. Ex: GA4 standard @javascript: document.title
    #[serde(rename = "dt")]
    pub document_title: String,
    /// Actual page's Referrer. https://www.edgee.dev/ga-standard.html @javascript: document.referrer
    #[serde(rename = "dr", skip_serializing_if = "Option::is_none")]
    pub document_referrer: Option<String>,
    /// Unknown. Value ccd.{{HASH}}. The hash in based on various internal parameters. Some kind of usage hash. Ex: ccd.AAB
    #[serde(rename = "_z", skip_serializing_if = "Option::is_none")]
    z: Option<String>,
    /// This is added when an event is generated from rules (from the admin) . Actually is hash of the "GA4_EVENT" string
    #[serde(rename = "_eu", skip_serializing_if = "Option::is_none")]
    event_usage: Option<String>,
    /// Unknown
    #[serde(rename = "edid", skip_serializing_if = "Option::is_none")]
    event_debug_id: Option<String>,
    /// If an event contains this parameters it won't be processed and it will show on on the debug View in GA4. Ex: 1
    #[serde(rename = "_dbg", skip_serializing_if = "Option::is_none")]
    is_debug: Option<String>,
    /// If the current request has a referrer, it will be ignored at processing level. Ex: 1
    #[serde(rename = "ir", skip_serializing_if = "Option::is_none")]
    ignore_referrer: Option<String>,
    /// Traffic Type. Ex: 1
    #[serde(rename = "tt", skip_serializing_if = "Option::is_none")]
    traffic_type: Option<String>,
    /// Will be set to 1 is the current page has a linker and this last one is valid. Ex: 1
    #[serde(rename = "_glv", skip_serializing_if = "Option::is_none")]
    is_google_linker_valid: Option<String>,

    // Campaign Attribution are directly grabbed from the URL
    // Those parameters will override the current values read from the URL, so they are clearly optional
    /// Campaign Medium ( utm_medium ), this will override the current values read from the url. Ex: cpc
    #[serde(rename = "cm", skip_serializing_if = "Option::is_none")]
    campaign_medium: Option<String>,
    /// Campaign Source ( utm_source ), this will override the current values read from the url. Ex: google
    #[serde(rename = "cs", skip_serializing_if = "Option::is_none")]
    campaign_source: Option<String>,
    /// Campaign Name ( utm_campaign ), this will override the current values read from the url. Ex: cpc
    #[serde(rename = "cn", skip_serializing_if = "Option::is_none")]
    campaign_name: Option<String>,
    /// Campaign Content ( utm_content ), this will override the current values read from the url. Ex: big banner
    #[serde(rename = "cc", skip_serializing_if = "Option::is_none")]
    campaign_content: Option<String>,
    /// Campaign Term ( utm_term ), this will override the current values read from the url. Ex: summer
    #[serde(rename = "ck", skip_serializing_if = "Option::is_none")]
    campaign_term: Option<String>,
    /// Campaign Creative Format ( utm_creative_format ), this will override the current values read from the url. Ex: native
    #[serde(rename = "ccf", skip_serializing_if = "Option::is_none")]
    campaign_creative_format: Option<String>,
    /// Campaign Marketing Tactic ( utm_marketing_tactic ), this will override the current values read from the url. Ex: remarketing
    #[serde(rename = "cmt", skip_serializing_if = "Option::is_none")]
    campaign_marketing_tactic: Option<String>,
    /// Random number used to dedupe gclid. Ex: 342342343
    #[serde(rename = "_rnd", skip_serializing_if = "Option::is_none")]
    gclid_deduper: Option<String>,

    // Event Parameters
    /// Current Event Name. Limits, 40 characters name length, 100 characters value length, 500 distinct event names by instance. Ex: page_view
    #[serde(rename = "en")]
    event_name: String,
    /// It's the total engagement time in milliseconds since the last event. The engagement time is measured only when the current page is visible and active ( ie: the browser window/tab must be active and visible ), for this GA4 uses the window.events: focus, blur, pageshow, pagehide and the document:visibilitychange, these will determine when the timer starts and pauses. Ex: 1234
    #[serde(rename = "_et", skip_serializing_if = "Option::is_none")]
    pub engagement_time: Option<String>,

    // Dynamic parameter handling is more complex in Rust and might require a custom deserializer
    /// Defines a parameter for the current Event with ep.* semantic. Ex: ep.page_type: checkout
    /// For ecommerce, the following parameters are used:
    /// - &ep.affiliation
    #[serde(rename = "ep", skip_serializing_if = "Option::is_none")]
    pub event_parameter_string: Option<HashMap<String, String>>,

    /// Defines a parameter for the current Event with epn.* semantic. Ex: epn.page_number: 123
    /// For ecommerce, the following parameters are used:
    /// - Transaction revenue: &epn.value
    /// - Transaction tax: &epn.tax
    /// - Transaction shipping: &epn.shipping
    #[serde(rename = "epn", skip_serializing_if = "Option::is_none")]
    pub event_parameter_number: Option<HashMap<String, f64>>,

    /// If the current event is set as a conversion on the admin interace the evfent will have this value present. Ex: 1
    #[serde(rename = "_c", skip_serializing_if = "Option::is_none")]
    is_conversion: Option<String>,
    /// External Event. Ex: 1
    #[serde(rename = "_ee", skip_serializing_if = "Option::is_none")]
    external_event: Option<String>,

    // Session / User Related
    /// Current User Id. Limit 256 characters. Ex: 123456789
    #[serde(rename = "uid", skip_serializing_if = "Option::is_none")]
    pub(crate) user_id: Option<String>,
    /// Current Firebase Id. Limit 256 characters. Ex: HASHSAH
    #[serde(rename = "_fid", skip_serializing_if = "Option::is_none")]
    firebase_id: Option<String>,
    /// GA4 Session Id. This comes from the GA4 Cookie. It may be different for each Stream ID Configured on the site. Ex: 123456789 @javascript: ('; ' + document.cookie).split('; _ga_XXX=').pop().split(';').shift().split('.')[2]
    #[serde(rename = "sid", skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    /// Count of sessions recorded by GA4. This value increases by one each time a new session is detected ( when the session expires ). Ex: 1 @javascript: ('; ' + document.cookie).split('; _ga_XXX=').pop().split(';').shift().split('.')[3]
    #[serde(rename = "sct", skip_serializing_if = "Option::is_none")]
    session_count: Option<String>,
    /// If the current user is engaged in any way, this value will be 1. Ex: 1
    #[serde(rename = "seg", skip_serializing_if = "Option::is_none")]
    session_engagement: Option<String>,
    /// Defines an user Propery (string) for the current Measurement ID. Ex: up.user_type: premium
    #[serde(rename = "up", skip_serializing_if = "Option::is_none")]
    pub(crate) user_property_string: Option<HashMap<String, String>>,
    /// Defines an user Propery (number) for the current Measurement ID. Ex: upn.lifetime_value: 45.50
    #[serde(rename = "upn", skip_serializing_if = "Option::is_none")]
    pub(crate) user_property_number: Option<HashMap<String, f64>>,
    /// If the "_ga_XXX" cookie is not set, the first event will have this value present. This will internally create a new "first_visit" event on GA4. If this event is also a conversion the value will be "2" if not, will be "1". Ex: 1
    #[serde(rename = "_fv", skip_serializing_if = "Option::is_none")]
    first_visit: Option<String>,
    /// If the "_ga_XXX" cookie last session time value is older than 1800 seconds, the current event will have this value present. This will internally create a new "session_start" event on GA4. If this event is also a conversion the value will be "2" if not, will be "1". Ex: 1
    #[serde(rename = "_ss", skip_serializing_if = "Option::is_none")]
    session_start: Option<String>,
    /// This seems to be related to the ServerSide hits, it's 0 if the FPLC Cookie is not present and to the current value if it's comming from a Cross Domain linker. Ex: 0
    #[serde(rename = "_fplc", skip_serializing_if = "Option::is_none")]
    first_party_linker_cookie: Option<String>,
    /// If the current user has a GA4 session cookie, but not a GA (_ga) client id cookie, this parameter will be added to the hit. Ex: 1
    #[serde(rename = "_nsi", skip_serializing_if = "Option::is_none")]
    new_session_id: Option<String>,
    /// You may find this parameter if using some vendor plugin o platform ( ie: using shopify integration or a prestashop plugin ). Ex: hjkds85
    #[serde(rename = "_gdid", skip_serializing_if = "Option::is_none")]
    google_developer_id: Option<String>,
    /// Added to report the current country for the user under some circumstanced. To be documented. Ex: US
    #[serde(rename = "_uc", skip_serializing_if = "Option::is_none")]
    user_country: Option<String>,

    // Google Consent mode V1
    /// Current Google Consent Status. Format 'G1'+'AdsStorageBoolStatus'`+'AnalyticsStorageBoolStatus'. Ex: G101.
    #[serde(rename = "gcs", skip_serializing_if = "Option::is_none")]
    google_consent_status: Option<String>,
    /// Will be added with the value "1" if the Google Consent has just been updated (wait_for_update setting on GTAG). Ex: 1
    #[serde(rename = "gcu", skip_serializing_if = "Option::is_none")]
    google_consent_update: Option<String>,
    /// Documented values, 1 or 2, no more info on the meaning. Ex: 1
    #[serde(rename = "gcut", skip_serializing_if = "Option::is_none")]
    google_consent_update_type: Option<String>,

    // Google Consent mode V2
    /// Google Consent Decoded.
    /// format xx{{ad_storage}}x{{analytics_storage}}x{{ad_user_data}}x{{ad_personalization}}xxx
    /// The {{consent signals}} values can be as follows:
    ///
    /// "p": value ‘denied’ by ‘default’ (the user has not yet made a choice)
    /// "t": ‘granted’ value by ‘default’ (the user has not yet made a choice)
    /// "q", ‘m’ or ‘u’: ‘denied’ value on update (user choice)
    /// "e", ‘r’, ‘n’ or ‘v’: value ‘granted’ during an update (user choice)
    /// "l": the signal has not been defined (this is the case if GCM v2 is not in place).
    ///
    /// ex: 13p3t3p2p5l1 (denied to all, but granted to analytics)
    /// 13t3t3t2t5l1 (granted to all)
    #[serde(rename = "gcd", skip_serializing_if = "Option::is_none")]
    gcd: Option<String>,
    /// Non personalized ads (0 or 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    npa: Option<String>,
    /// Consent to Google services is encoded using dma_cps: - (no consent), syphamo (s for Search, y for Youtube, p for Play, h for Shopping, a for Ad services, m for Maps, o for Other)
    #[serde(skip_serializing_if = "Option::is_none")]
    dma_cps: Option<String>,
    /// Documented values, 1 or 2, no more info on the meaning. Ex: 1
    #[serde(skip_serializing_if = "Option::is_none")]
    dma: Option<String>,
    /// Privacy Sandbox Cookie Deprecation Label. Ex: noapi or denied
    #[serde(skip_serializing_if = "Option::is_none")]
    pscdl: Option<String>,

    // Miscellaneous
    /// Tag Explorer. Ex: 101823848~101925629
    #[serde(skip_serializing_if = "Option::is_none")]
    tag_exp: Option<String>,
    /// Documented values, 0 or 1, no more info on the meaning. Ex: 1
    #[serde(skip_serializing_if = "Option::is_none")]
    are: Option<String>,
    /// Documented values, 0 or 1, no more info on the meaning. Ex: 1
    #[serde(skip_serializing_if = "Option::is_none")]
    pae: Option<String>,
    /// Documented values, 0 or 1, no more info on the meaning. Ex: 0
    #[serde(skip_serializing_if = "Option::is_none")]
    frm: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    ec_mode: Option<String>, // ex: c

    /// Timestamp measuring the difference between the moment this parameter gets populated and the moment the navigation started on that particular page. Calculated in JS with Math.round(window.performance.now())
    #[serde(skip_serializing_if = "Option::is_none")]
    tfd: Option<String>,

    /// Ecommerce
    /// Currency Code. ISO 4217
    #[serde(rename = "cu", skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,

    /// IP Override
    #[serde(rename = "_uip", skip_serializing_if = "Option::is_none")]
    pub ip_override: Option<String>,
}

/// Product (item). Converted into a GA4 string, it contains an item details and all it's params.
/// Example Value: &pr1=id123456~nmTshirtbrThyngster~camen~c2shirts~pr129.99~k0currency~v0JPY~k1stock~v1yes
#[derive(Serialize, Debug, Default)]
pub struct Product {
    // items parameters (An event can only hold up to 200 items details. Any items above that limit will be removed from the payload)
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>, // 12345
    #[serde(rename = "nm", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>, // Tshirt
    #[serde(rename = "af", skip_serializing_if = "Option::is_none")]
    pub affiliation: Option<String>, // Google Store
    #[serde(rename = "cp", skip_serializing_if = "Option::is_none")]
    pub coupon: Option<String>, // SUMMER2019
    #[serde(rename = "ds", skip_serializing_if = "Option::is_none")]
    pub discount: Option<String>, // 5.00
    #[serde(rename = "lp", skip_serializing_if = "Option::is_none")]
    pub index: Option<String>, // 0

    #[serde(rename = "br", skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>, // Google
    #[serde(rename = "ca", skip_serializing_if = "Option::is_none")]
    pub category: Option<String>, // Electronics
    #[serde(rename = "c2", skip_serializing_if = "Option::is_none")]
    pub category2: Option<String>, // Accessories
    #[serde(rename = "c3", skip_serializing_if = "Option::is_none")]
    pub category3: Option<String>, // Cables
    #[serde(rename = "c4", skip_serializing_if = "Option::is_none")]
    pub category4: Option<String>, // HDMI
    #[serde(rename = "c5", skip_serializing_if = "Option::is_none")]
    pub category5: Option<String>, // HDMI
    #[serde(rename = "li", skip_serializing_if = "Option::is_none")]
    pub list_id: Option<String>, // SR123
    #[serde(rename = "ln", skip_serializing_if = "Option::is_none")]
    pub list_name: Option<String>, // Search Results
    #[serde(rename = "va", skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>, // 8ft

    #[serde(rename = "lo", skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>, // ChIJIQBpAG2ahYAR_6128GcTUEo

    #[serde(rename = "pr", skip_serializing_if = "Option::is_none")]
    pub price: Option<String>, // 19.99
    #[serde(rename = "qt", skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>, // 1

    // custom parameters
    // each parameter is a key/value pair
    // ex: in_stock: true, color: green
    // will be converted into a k0in_stock~v0true~k1color~v1green
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_parameters: Option<Vec<(String, String)>>,
}

impl GaPayload {
    pub(crate) fn new(
        edgee_event: &Event,
        settings: Dict,
        event_name: String,
    ) -> anyhow::Result<Self> {
        let cred: HashMap<String, String> = settings
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        let measurement_id = match cred.get("ga_measurement_id") {
            Some(v) => v,
            None => return Err(anyhow!("Missing GA Measurement ID")),
        }
        .to_string();

        let mut ga = GaPayload {
            protocol_version: "2".to_string(),
            tracking_id: measurement_id,
            event_name,
            random_page_load_hash: random_page_load_hash(),
            external_event: Some("1".to_string()),
            ..GaPayload::default()
        };

        // page
        if !edgee_event.context.page.title.is_empty() {
            ga.document_title = edgee_event.context.page.title.clone();
        }
        if !edgee_event.context.page.url.is_empty() {
            let document_location = format!(
                "{}{}",
                edgee_event.context.page.url.clone(),
                edgee_event.context.page.search.clone()
            );
            ga.document_location = document_location;
        }
        if !edgee_event.context.page.referrer.is_empty() {
            ga.document_referrer = Some(edgee_event.context.page.referrer.clone());
        }

        if edgee_event.consent.is_some() && edgee_event.consent.unwrap() == Consent::Granted {
            // Consent is fully granted
            ga.google_consent_status = Some("G111".to_string());
            ga.gcd = Some("13t3t3t2t5l1".to_string());
            ga.npa = Some("0".to_string());
            ga.dma_cps = Some("syphamo".to_string());
            ga.dma = Some("1".to_string());
            ga.pscdl = Some("noapi".to_string());
        } else {
            // Consent is set to analytics only
            ga.google_consent_status = Some("G101".to_string());
            ga.gcd = Some("13p3t3p2p5l1".to_string());
            ga.npa = Some("1".to_string());
            ga.dma_cps = Some("-".to_string());
            ga.dma = Some("1".to_string());
            ga.pscdl = Some("denied".to_string());
        }

        // forge the typical ga ClientId
        let first_seen = edgee_event.context.session.first_seen;

        // if edgee_id is a uuid, convert it to a 9 digit string
        let ga_client_id = if is_valid_uuid(edgee_event.context.user.edgee_id.clone().as_str()) {
            let nine_digit_id = uuid_to_nine_digit_string(&edgee_event.context.user.edgee_id)?;
            format!("{}.{}", nine_digit_id, first_seen)
        } else {
            edgee_event.context.user.edgee_id.clone()
        };
        ga.client_id = ga_client_id;

        ga.hit_counter = "1".to_string();

        if edgee_event.context.client.locale.is_empty() {
            ga.user_language = "en".to_string();
        } else {
            ga.user_language = edgee_event.context.client.locale.clone();
        }
        if !edgee_event
            .context
            .client
            .user_agent_full_version_list
            .is_empty()
        {
            ga.user_agent_full_version_list = Some(
                edgee_event
                    .context
                    .client
                    .user_agent_full_version_list
                    .clone(),
            );
        }
        if !edgee_event.context.client.user_agent_mobile.is_empty() {
            ga.user_agent_mobile = Some(edgee_event.context.client.user_agent_mobile.clone());
        }
        if !edgee_event.context.client.os_name.is_empty() {
            ga.user_agent_platform = Some(edgee_event.context.client.os_name.clone());
        }
        if !edgee_event.context.client.os_version.is_empty() {
            ga.user_agent_platform_version = Some(edgee_event.context.client.os_version.clone());
        }
        if !edgee_event
            .context
            .client
            .user_agent_architecture
            .is_empty()
        {
            ga.user_agent_architecture =
                Some(edgee_event.context.client.user_agent_architecture.clone());
        }
        if !edgee_event.context.client.user_agent_bitness.is_empty() {
            ga.user_agent_bitness = Some(edgee_event.context.client.user_agent_bitness.clone());
        }
        if !edgee_event.context.client.user_agent_model.is_empty() {
            ga.user_agent_model = Some(edgee_event.context.client.user_agent_model.clone());
        }

        if edgee_event.context.client.screen_width.is_positive()
            && edgee_event.context.client.screen_height.is_positive()
        {
            ga.screen_resolution = Some(format!(
                "{:?}x{:?}",
                edgee_event.context.client.screen_width.clone(),
                edgee_event.context.client.screen_height.clone()
            ));
        }

        // user
        let mut user_property_string: HashMap<String, String> = HashMap::new();
        let mut user_property_number: HashMap<String, f64> = HashMap::new();
        if !edgee_event.context.user.anonymous_id.is_empty() {
            ga.user_id = Some(edgee_event.context.user.anonymous_id.clone());
        }
        if !edgee_event.context.user.user_id.is_empty() {
            ga.user_id = Some(edgee_event.context.user.user_id.clone());
            if !edgee_event.context.user.anonymous_id.is_empty() {
                user_property_string.insert(
                    "anonymous_id".to_string(),
                    edgee_event.context.user.anonymous_id.clone(),
                );
            }
        }

        // user properties
        if !edgee_event.context.user.properties.is_empty() {
            for (key, value) in edgee_event.context.user.properties.clone().iter() {
                // if key has a space, replace by a _
                let key = key.replace(" ", "_");
                if value.parse::<f64>().is_ok() {
                    user_property_number.insert(key, value.parse().unwrap());
                } else {
                    user_property_string.insert(key, value.clone());
                }
            }
        }

        if !user_property_string.is_empty() {
            ga.user_property_string = Some(user_property_string);
        }
        if !user_property_number.is_empty() {
            ga.user_property_number = Some(user_property_number);
        }

        // geo ip
        if !edgee_event.context.client.country_code.is_empty() {
            ga.user_country = Some(edgee_event.context.client.country_code.clone());
        }

        // ip override
        if !edgee_event.context.client.ip.is_empty() {
            ga.ip_override = Some(edgee_event.context.client.ip.clone());
        }

        // Campaign are directly grabbed by GA from the URL
        // There's no need to set them here
        // if !edgee_event.context.campaign.medium.is_empty() {
        //     ga.campaign_medium = Some(edgee_event.context.campaign.medium.clone());
        // }
        // if !edgee_event.context.campaign.source.is_empty() {
        //     ga.campaign_source = Some(edgee_event.context.campaign.source.clone());
        // }
        // if !edgee_event.context.campaign.name.is_empty() {
        //     ga.campaign_name = Some(edgee_event.context.campaign.name.clone());
        // }
        // if !edgee_event.context.campaign.content.is_empty() {
        //     ga.campaign_content = Some(edgee_event.context.campaign.content.clone());
        // }
        // if !edgee_event.context.campaign.term.is_empty() {
        //     ga.campaign_term = Some(edgee_event.context.campaign.term.clone());
        // }
        // if !edgee_event.context.page.search.is_empty() {
        //     // analyze search string
        //     let qs = serde_qs::from_str(edgee_event.context.page.search.as_str());
        //     if qs.is_ok() {
        //         let qs_map: HashMap<String, String> = qs.unwrap();
        //         for (key, value) in qs_map.iter() {
        //             if key.eq("utm_creative_format") {
        //                 ga.campaign_creative_format = Some(value.clone());
        //             }
        //             if key.eq("utm_marketing_tactic") {
        //                 ga.campaign_marketing_tactic = Some(value.clone());
        //             }
        //         }
        //     }
        // }

        // session
        ga.session_id = Some(edgee_event.context.session.session_id.clone());
        ga.session_count = Some(
            edgee_event
                .context
                .session
                .session_count
                .clone()
                .to_string(),
        );

        if edgee_event.context.session.first_seen == edgee_event.context.session.last_seen {
            ga.first_visit = Some(String::from("1"));
            ga.new_session_id = Some(String::from("1"));
        }

        // when a new session starts, ga4.SessionEngagement = 0, when it doesn't ga4.SessionEngagement = 1
        if edgee_event.context.session.session_start {
            ga.session_start = Some(String::from("1"));
            ga.session_engagement = Some(String::from("0"));
        } else {
            ga.session_engagement = Some(String::from("1"));
        }

        Ok(ga)
    }
}

fn random_page_load_hash() -> String {
    let now = Utc::now().timestamp_nanos_opt().unwrap(); // Get current time as a nanosecond timestamp
    let mut rng = StdRng::seed_from_u64(now as u64); // Seed the RNG with current time
    let random_number = rng.gen_range(0..=2147483647); // Generate a random number in the range [0, 2147483647]
    random_number.to_string() // Return the random number as a string
}

fn uuid_to_nine_digit_string(uuid: &str) -> anyhow::Result<String> {
    // Create an MD5 hasher instance
    let result = md5::compute(uuid.as_bytes());

    // Convert hash result to hex string
    let hash_hex = format!("{:x}", result);

    // Convert hex string to a big integer
    let mut hash_bigint = BigInt::from_str_radix(&hash_hex, 16)?;

    // Calculate the modulo to limit the number to 9 digits
    let modulo = 1000000000.to_bigint().unwrap();
    hash_bigint %= modulo;

    // if the number is not 9 digits, add leading ones
    let hash_bigint = if hash_bigint.to_string().len() < 9 {
        let leading_ones = "1".repeat(9 - hash_bigint.to_string().len());
        format!("{}{}", leading_ones, hash_bigint)
    } else {
        hash_bigint.to_string()
    };

    // Return the result as a string
    Ok(hash_bigint.to_string())
}

fn is_valid_uuid(uuid_str: &str) -> bool {
    match uuid::Uuid::parse_str(uuid_str) {
        Ok(uuid_obj) => uuid_obj.get_version_num() == 4,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_to_nine_digit_string_eq() {
        let input = "00000000-0000-0000-0000-000000000000";
        let result = uuid_to_nine_digit_string(input).unwrap();
        assert_eq!(result, "151760947");
    }

    #[test]
    fn uuid_to_nine_digit_string_eq2() {
        let input = "be9f76b3-2c50-4d12-b14c-85c343745691";
        let result = uuid_to_nine_digit_string(input).unwrap();
        assert_eq!(result, "108670052");
    }

    #[test]
    fn uuid_to_nine_digit_string_valid_uuid() {
        let input = uuid::Uuid::new_v4().to_string();
        let result = uuid_to_nine_digit_string(input.as_str()).unwrap();
        assert_eq!(result.len(), 9);
    }

    #[test]
    fn uuid_to_nine_digit_string_valid_uuid2() {
        let input = uuid::Uuid::new_v4().to_string();
        let result = uuid_to_nine_digit_string(input.as_str()).unwrap();
        assert_eq!(result.len(), 9);
    }

    #[test]
    fn uuid_to_nine_digit_string_valid_uuid3() {
        let input = uuid::Uuid::new_v4().to_string();
        let result = uuid_to_nine_digit_string(input.as_str()).unwrap();
        assert_eq!(result.len(), 9);
    }

    #[test]
    fn uuid_to_nine_digit_string_valid_uuid4() {
        let input = uuid::Uuid::new_v4().to_string();
        let result = uuid_to_nine_digit_string(input.as_str()).unwrap();
        assert_eq!(result.len(), 9);
    }

    #[test]
    fn uuid_to_nine_digit_string_invalid_uuid() {
        let input = "invalid-uuid";
        let result = uuid_to_nine_digit_string(input).unwrap();
        assert_eq!(result.len(), 9);
    }

    #[test]
    fn random_page_load_hash_length() {
        let result = random_page_load_hash();
        assert!(!result.is_empty());
    }

    #[test]
    fn random_page_load_hash_is_numeric() {
        let result = random_page_load_hash();
        assert!(result.chars().all(|c| c.is_numeric()));
    }

    #[test]
    fn random_page_load_hash_is_within_range() {
        let result = random_page_load_hash();
        let number: i64 = result.parse().unwrap();
        assert!((0..=2147483647).contains(&number));
    }
}
