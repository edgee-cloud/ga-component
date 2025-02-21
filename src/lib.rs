use crate::exports::edgee::components::data_collection::{
    Data, Dict, EdgeeRequest, Event, HttpMethod,
};
use exports::edgee::components::data_collection::Guest;
use ga_payload::{GaPayload, Product};
use std::collections::HashMap;
mod ga_payload;

wit_bindgen::generate!({world: "data-collection", path: "wit", generate_all});
export!(GaComponent);

struct GaComponent;

impl Guest for GaComponent {
    fn page(edgee_event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Page(ref data) = edgee_event.data {
            let mut ga = GaPayload::new(&edgee_event, settings, "page_view".to_string())
                .map_err(|e| e.to_string())?;

            let document_location = format!("{}{}", data.url.clone(), data.search.clone());
            ga.document_location = document_location;
            ga.document_title = data.title.clone();
            ga.document_referrer = Some(data.referrer.clone());

            let mut event_parameter_string = HashMap::new();
            event_parameter_string.insert("event_id".to_string(), edgee_event.uuid.clone());

            let mut event_parameter_number = HashMap::new();

            if !data.name.is_empty() {
                event_parameter_string.insert("page_name".to_string(), data.name.clone());
            }

            if !data.category.is_empty() {
                event_parameter_string.insert("page_category".to_string(), data.category.clone());
            }

            if !data.keywords.is_empty() {
                event_parameter_string.insert("page_keywords".to_string(), data.keywords.join(","));
            }

            if !data.search.is_empty() {
                event_parameter_string.insert("page_search".to_string(), data.search.clone());
            }

            for (key, value) in data.properties.iter() {
                let key = key.replace(" ", "_");
                if value.parse::<f64>().is_ok() {
                    event_parameter_number.insert(key, value.parse().unwrap());
                } else if key == "currency" {
                    ga.currency_code = Some(value.clone());
                } else {
                    event_parameter_string.insert(key, value.clone());
                }
            }

            ga.event_parameter_string = Some(event_parameter_string);
            if !event_parameter_number.is_empty() {
                ga.event_parameter_number = Some(event_parameter_number);
            }

            Ok(build_edgee_request(ga, vec![]).map_err(|e| e.to_string())?)
        } else {
            Err("Missing page data".to_string())
        }
    }

    fn track(edgee_event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Track(ref data) = edgee_event.data {
            if data.name.is_empty() {
                return Err("Track is not set".to_string());
            }

            let mut ga = GaPayload::new(&edgee_event, settings, data.name.clone())
                .map_err(|e| e.to_string())?;

            let mut event_parameter_string = HashMap::new();
            event_parameter_string.insert("event_id".to_string(), edgee_event.uuid.clone());

            let mut event_parameter_number = HashMap::new();

            for (key, value) in data.properties.iter() {
                let key = key.replace(" ", "_");
                if value.parse::<f64>().is_ok() {
                    event_parameter_number.insert(key, value.parse().unwrap());
                } else if key == "currency" {
                    ga.currency_code = Some(value.clone());
                } else {
                    event_parameter_string.insert(key, value.clone());
                }
            }

            ga.event_parameter_string = Some(event_parameter_string);
            if !event_parameter_number.is_empty() {
                ga.event_parameter_number = Some(event_parameter_number);
            }

            let ga_items: Vec<Product> = data
                .products
                .iter()
                .map(|product| {
                    let mut p = Product::default();
                    let mut p_custom_params = Vec::new();

                    for (key, value) in product.iter() {
                        let key = key.replace(" ", "_");
                        match key.as_str() {
                            "sku" => p.sku = Some(value.clone()),
                            "name" => p.name = Some(value.clone()),
                            "affiliation" => p.affiliation = Some(value.clone()),
                            "coupon" => p.coupon = Some(value.clone()),
                            "discount" => p.discount = Some(value.clone()),
                            "index" => p.index = Some(value.clone()),
                            "brand" => p.brand = Some(value.clone()),
                            "category" => p.category = Some(value.clone()),
                            "category2" => p.category2 = Some(value.clone()),
                            "category3" => p.category3 = Some(value.clone()),
                            "category4" => p.category4 = Some(value.clone()),
                            "category5" => p.category5 = Some(value.clone()),
                            "list_id" => p.list_id = Some(value.clone()),
                            "list_name" => p.list_name = Some(value.clone()),
                            "variant" => p.variant = Some(value.clone()),
                            "location_id" => p.location_id = Some(value.clone()),
                            "price" => p.price = Some(value.clone()),
                            "quantity" => p.quantity = Some(value.clone()),
                            _ => p_custom_params.push((key, value.clone())),
                        }
                    }

                    if !p_custom_params.is_empty() {
                        p.custom_parameters = Some(p_custom_params);
                    }
                    p
                })
                .collect();

            Ok(build_edgee_request(ga, ga_items).map_err(|e| e.to_string())?)
        } else {
            Err("Missing track data".to_string())
        }
    }

    fn user(_edgee_event: Event, _settings: Dict) -> Result<EdgeeRequest, String> {
        Err("User event is not mapped to any Google Analytics event".to_string())
    }
}

fn build_edgee_request(ga: GaPayload, ga_items: Vec<Product>) -> anyhow::Result<EdgeeRequest> {
    let headers = vec![(String::from("content-length"), String::from("0"))];

    let mut querystring = serde_qs::to_string(&ga)?;
    querystring = cleanup_querystring(&querystring)?;

    if !ga_items.is_empty() {
        // add the items to the querystring, to do so, each item is converted to a string and added to the querystring
        // an item starts with &pr following by the item number (for example &pr1, &pr2, &pr3, etc.)
        // then, the value of the item is one string with the item parameters separated by ~
        // example: &pr1=id123456~nmTshirtbrThyngster~camen~c2shirts~pr129.99~k0currency~v0JPY~k1stock~v1yes
        // then the string has to be urlencoded
        let mut item_strings = Vec::new();
        for (index, item) in ga_items.iter().enumerate() {
            let mut item_parts = Vec::new();

            if let Some(sku) = &item.sku {
                item_parts.push(format!("id{}", sku));
            }
            if let Some(name) = &item.name {
                item_parts.push(format!("nm{}", name));
            }
            if let Some(brand) = &item.brand {
                item_parts.push(format!("br{}", brand));
            }
            if let Some(category) = &item.category {
                item_parts.push(format!("ca{}", category));
            }
            if let Some(price) = &item.price {
                item_parts.push(format!("pr{}", price));
            }
            if let Some(affiliation) = &item.affiliation {
                item_parts.push(format!("af{}", affiliation));
            }
            if let Some(coupon) = &item.coupon {
                item_parts.push(format!("cp{}", coupon));
            }
            if let Some(discount) = &item.discount {
                item_parts.push(format!("ds{}", discount));
            }
            if let Some(index_val) = &item.index {
                item_parts.push(format!("lp{}", index_val));
            }
            if let Some(category2) = &item.category2 {
                item_parts.push(format!("c2{}", category2));
            }
            if let Some(category3) = &item.category3 {
                item_parts.push(format!("c3{}", category3));
            }
            if let Some(category4) = &item.category4 {
                item_parts.push(format!("c4{}", category4));
            }
            if let Some(category5) = &item.category5 {
                item_parts.push(format!("c5{}", category5));
            }
            if let Some(list_id) = &item.list_id {
                item_parts.push(format!("li{}", list_id));
            }
            if let Some(list_name) = &item.list_name {
                item_parts.push(format!("ln{}", list_name));
            }
            if let Some(variant) = &item.variant {
                item_parts.push(format!("va{}", variant));
            }
            if let Some(location_id) = &item.location_id {
                item_parts.push(format!("lo{}", location_id));
            }
            if let Some(quantity) = &item.quantity {
                item_parts.push(format!("qt{}", quantity));
            }

            // Add custom parameters if present
            if let Some(custom_params) = &item.custom_parameters {
                for (param_index, (key, value)) in custom_params.iter().enumerate() {
                    item_parts.push(format!("k{}{}", param_index, key));
                    item_parts.push(format!("v{}{}", param_index, value));
                }
            }

            // Join all parts with ~ and URL encode
            let item_value = urlencoding::encode(&item_parts.join("~")).into_owned();
            item_strings.push(format!("&pr{}={}", index + 1, item_value));
        }

        // Add all item strings to querystring
        let items_qs = item_strings.join("");
        querystring = format!("{}{}", querystring, items_qs);
    }

    Ok(EdgeeRequest {
        method: HttpMethod::Post,
        url: format!("https://www.google-analytics.com/g/collect?{}", querystring),
        headers,
        forward_client_headers: true,
        body: String::new(),
    })
}

// change every parameters like ep[page_type]=checkout to ep.page_type=checkout. check only the ep, epn, up, upn parameters
fn cleanup_querystring(ga4_qs: &str) -> anyhow::Result<String> {
    let mut cleaned_qs = String::new();
    let mut i = 0;
    let chars: Vec<char> = ga4_qs.chars().collect();
    let len = chars.len();

    while i < len {
        if i + 2 < len {
            // Check for "ep[", "epn[", "up[", or "upn[" patterns
            if (chars[i] == 'e' && chars[i + 1] == 'p' && chars[i + 2] == '[')
                || (chars[i] == 'e'
                    && chars[i + 1] == 'p'
                    && chars[i + 2] == 'n'
                    && i + 3 < len
                    && chars[i + 3] == '[')
                || (chars[i] == 'u' && chars[i + 1] == 'p' && chars[i + 2] == '[')
                || (chars[i] == 'u'
                    && chars[i + 1] == 'p'
                    && chars[i + 2] == 'n'
                    && i + 3 < len
                    && chars[i + 3] == '[')
            {
                // Append the base part of the key (e.g. "ep", "epn", "up", or "upn")
                let base = if chars[i] == 'e' {
                    if chars[i + 2] == 'n' {
                        "epn"
                    } else {
                        "ep"
                    }
                } else if chars[i + 2] == 'n' {
                    "upn"
                } else {
                    "up"
                };
                cleaned_qs.push_str(base);
                cleaned_qs.push('.'); // Replace "[" with "."

                // Skip the characters that form "ep[" or "epn["
                if base == "ep" || base == "up" {
                    i += 3; // Skip "ep[" or "up["
                } else {
                    i += 4; // Skip "epn[" or "upn["
                }

                // Capture the content between brackets and append
                while i < len && chars[i] != ']' {
                    cleaned_qs.push(chars[i]);
                    i += 1;
                }
                i += 1; // Skip the closing bracket ']'
                continue;
            }
        }

        // Add the current character to the output if no special pattern was found
        cleaned_qs.push(chars[i]);
        i += 1;
    }

    Ok(cleaned_qs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exports::edgee::components::data_collection::{
        Campaign, Client, Context, EventType, PageData, Session, TrackData, UserData,
    };
    use exports::edgee::components::data_collection::Consent;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    #[test]
    fn cleanup_querystring_replaces_correctly() {
        let input = "ep[page_type]=checkout&epn[page_number]=1&up[user_id]=123&upn[user_age]=30";
        let expected = "ep.page_type=checkout&epn.page_number=1&up.user_id=123&upn.user_age=30";
        let result = cleanup_querystring(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn cleanup_querystring_handles_empty_string() {
        let input = "";
        let expected = "";
        let result = cleanup_querystring(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn cleanup_querystring_handles_no_replacements() {
        let input = "some_param=value&another_param=value2";
        let expected = "some_param=value&another_param=value2";
        let result = cleanup_querystring(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn cleanup_querystring_handles_partial_replacements() {
        let input = "ep[page_type]=checkout&some_param=value";
        let expected = "ep.page_type=checkout&some_param=value";
        let result = cleanup_querystring(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn cleanup_querystring_handles_multiple_replacements() {
        let input = "ep[page_type]=checkout&ep[page_name]=home&up[user_id]=123";
        let expected = "ep.page_type=checkout&ep.page_name=home&up.user_id=123";
        let result = cleanup_querystring(input).unwrap();
        assert_eq!(result, expected);
    }

    fn sample_user_data(edgee_id: String) -> UserData {
        UserData {
            user_id: "123".to_string(),
            anonymous_id: "456".to_string(),
            edgee_id,
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
            ],
        }
    }

    fn sample_context(edgee_id: String, locale: String, session_start: bool) -> Context {
        Context {
            page: sample_page_data(),
            user: sample_user_data(edgee_id),
            client: Client {
                city: "Paris".to_string(),
                ip: "192.168.0.1".to_string(),
                locale,
                timezone: "CET".to_string(),
                user_agent: "Chrome".to_string(),
                user_agent_architecture: "fuck knows".to_string(),
                user_agent_bitness: "64".to_string(),
                user_agent_full_version_list: "abc".to_string(),
                user_agent_version_list: "abc".to_string(),
                user_agent_mobile: "mobile".to_string(),
                user_agent_model: "don't know".to_string(),
                os_name: "MacOS".to_string(),
                os_version: "latest".to_string(),
                screen_width: 1024,
                screen_height: 768,
                screen_density: 2.0,
                continent: "Europe".to_string(),
                country_code: "FR".to_string(),
                country_name: "France".to_string(),
                region: "West Europe".to_string(),
            },
            campaign: Campaign {
                name: "random".to_string(),
                source: "random".to_string(),
                medium: "random".to_string(),
                term: "random".to_string(),
                content: "random".to_string(),
                creative_format: "random".to_string(),
                marketing_tactic: "random".to_string(),
            },
            session: Session {
                session_id: "random".to_string(),
                previous_session_id: "random".to_string(),
                session_count: 2,
                session_start,
                first_seen: 123,
                last_seen: 123,
            },
        }
    }

    fn sample_page_data() -> PageData {
        PageData {
            name: "page name".to_string(),
            category: "category".to_string(),
            keywords: vec!["value1".to_string(), "value2".into()],
            title: "page title".to_string(),
            url: "https://example.com/full-url?test=1".to_string(),
            path: "/full-path".to_string(),
            search: "?test=1".to_string(),
            referrer: "https://example.com/another-page".to_string(),
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
                ("currency".to_string(), "USD".to_string()),
            ],
        }
    }

    fn sample_page_event(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::Page,
            data: Data::Page(sample_page_data()),
            context: sample_context(edgee_id, locale, session_start),
            consent,
        }
    }

    fn sample_track_data(event_name: String) -> TrackData {
        TrackData {
            name: event_name,
            products: vec![
                vec![("sku".to_string(), "SKU_12345".to_string())],
                vec![("name".to_string(), "Stan and Friends Tee".to_string())],
                vec![(
                    "affiliation".to_string(),
                    "Google Merchandise Store".to_string(),
                )],
                vec![("coupon".to_string(), "SUMMER_FUN".to_string())],
                vec![("discount".to_string(), "2.22".to_string())],
                vec![("index".to_string(), "0".to_string())],
                vec![("brand".to_string(), "Google".to_string())],
                vec![("category".to_string(), "Apparel".to_string())],
                vec![("category2".to_string(), "Adult".to_string())],
                vec![("category3".to_string(), "Shirts".to_string())],
                vec![("category4".to_string(), "Crew".to_string())],
                vec![("category5".to_string(), "Short sleeve".to_string())],
                vec![("list_id".to_string(), "related_products".to_string())],
                vec![("list_name".to_string(), "Related Products".to_string())],
                vec![("variant".to_string(), "green".to_string())],
                vec![(
                    "location_id".to_string(),
                    "ChIJIQBpAG2ahYAR_6128GcTUEo".to_string(),
                )],
                vec![("price".to_string(), "10.1".to_string())],
                vec![("quantity".to_string(), "3".to_string())],
                vec![("custom-property".to_string(), "whatever".to_string())],
            ],
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
                ("currency".to_string(), "USD".to_string()),
            ],
        }
    }

    fn sample_track_event(
        event_name: String,
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::Track,
            data: Data::Track(sample_track_data(event_name)),
            context: sample_context(edgee_id, locale, session_start),
            consent,
        }
    }

    fn sample_user_event(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::User,
            data: Data::User(sample_user_data(edgee_id.clone())),
            context: sample_context(edgee_id, locale, session_start),
            consent,
        }
    }

    fn sample_settings() -> Vec<(String, String)> {
        vec![("ga_measurement_id".to_string(), "abc".to_string())]
    }

    #[test]
    fn page_with_consent() {
        let event = sample_page_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = GaComponent::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len(), 0);
        assert_eq!(
            edgee_request
                .url
                .starts_with("https://www.google-analytics.com"),
            true
        );
        // add more checks (headers, querystring, etc.)
    }

    #[test]
    fn page_without_consent() {
        let event = sample_page_event(None, "abc".to_string(), "fr".to_string(), true);
        let settings = sample_settings();
        let result = GaComponent::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len(), 0);
    }

    #[test]
    fn page_with_edgee_id_uuid() {
        let event = sample_page_event(None, Uuid::new_v4().to_string(), "fr".to_string(), true);
        let settings = sample_settings();
        let result = GaComponent::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len(), 0);
    }

    #[test]
    fn page_with_empty_locale() {
        let event = sample_page_event(None, Uuid::new_v4().to_string(), "".to_string(), true);

        let settings = sample_settings();
        let result = GaComponent::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len(), 0);
    }

    #[test]
    fn page_not_session_start() {
        let event = sample_page_event(None, Uuid::new_v4().to_string(), "".to_string(), false);
        let settings = sample_settings();
        let result = GaComponent::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len(), 0);
    }

    #[test]
    fn page_without_measurement_id_fails() {
        let event = sample_page_event(None, "abc".to_string(), "fr".to_string(), true);
        let settings: Vec<(String, String)> = vec![]; // empty
        let result = GaComponent::page(event, settings); // this should panic!
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn track_with_consent() {
        let event = sample_track_event(
            "event-name".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = GaComponent::track(event, settings);
        assert_eq!(result.clone().is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len(), 0);
    }

    #[test]
    fn track_with_empty_name_fails() {
        let event = sample_track_event(
            "".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = GaComponent::track(event, settings);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn user_event() {
        let event = sample_user_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = GaComponent::user(event, settings);

        assert_eq!(result.clone().is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("not mapped"),
            true
        );
    }

    #[test]
    fn track_event_without_user_context_properties_and_empty_user_id() {
        let mut event = sample_track_event(
            "event-name".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        event.context.user.properties = vec![]; // empty context user properties
        event.context.user.user_id = "".to_string(); // empty context user id
        let settings = sample_settings();
        let result = GaComponent::track(event, settings);
        //println!("Error: {}", result.clone().err().unwrap().to_string().as_str());
        assert_eq!(result.clone().is_err(), false);
    }

    /*

    fn sample_page_event_wrong_event_type() -> Event {
        return Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::User, // this is wrong!
            data: Data::Page(sample_page_data()),
            context: sample_context("abc".to_string(), "fr".to_string(), false),
            consent: Some(Consent::Granted),
        };
    }

    #[test]
    fn page_with_wrong_event_type() {
        // THIS TEST SHOULD FAIL BUT IT WORKS FINE =)
        let event = sample_page_event_wrong_event_type();
        let settings = sample_settings();
        let result = GaComponent::page(event, settings);
        assert_eq!(result.is_err(), true);
    }
     */
}
