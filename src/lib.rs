use exports::provider::{Data, Dict, EdgeeRequest, Event, Guest};
use ga_payload::GaPayload;
use std::collections::HashMap;

mod ga_payload;

wit_bindgen::generate!({world: "data-collection"});
export!(GaComponent);

struct GaComponent;

impl Guest for GaComponent {
    fn page(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Page(ref data) = edgee_event.data {
            let mut ga = GaPayload::new(&edgee_event, cred_map, "page_view".to_string())
                .map_err(|e| e.to_string())?;
            ga.document_location = data.url.clone();
            ga.document_title = data.title.clone();
            ga.document_referrer = Some(data.referrer.clone());

            let mut event_parameter_string = HashMap::new();
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
                if let Some(value) = value.parse::<f64>().ok() {
                    event_parameter_number.insert(key, value);
                } else {
                    event_parameter_string
                        .insert(key, value.clone());
                }
            }

            if event_parameter_string.len() > 0 {
                ga.event_parameter_string = Some(event_parameter_string);
            }
            if event_parameter_number.len() > 0 {
                ga.event_parameter_number = Some(event_parameter_number);
            }

            Ok(build_edgee_request(ga).map_err(|e| e.to_string())?)
        } else {
            Err("Missing page data".to_string())
        }
    }

    fn track(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Track(ref data) = edgee_event.data {
            if data.name.is_empty() {
                return Err("Track is not set".to_string());
            }

            let mut ga = GaPayload::new(&edgee_event, cred_map, String::from(data.name.clone()))
                .map_err(|e| e.to_string())?;

            let mut event_parameter_string = HashMap::new();
            let mut event_parameter_number = HashMap::new();

            for (key, value) in data.properties.iter() {
                let key = key.replace(" ", "_");
                if let Some(value) = value.parse::<f64>().ok() {
                    event_parameter_number.insert(key, value);
                } else {
                    event_parameter_string
                        .insert(key, value.clone());
                }
            }

            if event_parameter_string.len() > 0 {
                ga.event_parameter_string = Some(event_parameter_string);
            }
            if event_parameter_number.len() > 0 {
                ga.event_parameter_number = Some(event_parameter_number);
            }

            Ok(build_edgee_request(ga).map_err(|e| e.to_string())?)
        } else {
            Err("Missing track data".to_string())
        }
    }

    fn user(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if let Data::User(ref data) = edgee_event.data {
            if data.user_id.is_empty() && data.anonymous_id.is_empty() {
                return Err("user_id or anonymous_id is not set".to_string());
            }

            let mut ga = GaPayload::new(&edgee_event, cred_map, "user".to_string())
                .map_err(|e| e.to_string())?;

            // override the user data with the event.data fields
            let mut user_property_string: HashMap<String, String> = HashMap::new();
            let mut user_property_number: HashMap<String, f64> = HashMap::new();
            if !data.anonymous_id.is_empty() {
                ga.user_id = Some(data.anonymous_id.clone());
            }
            if !data.user_id.is_empty() {
                ga.user_id = Some(data.user_id.clone());
                if !data.anonymous_id.is_empty() {
                    user_property_string
                        .insert("anonymous_id".to_string(), data.anonymous_id.clone());
                }
            }

            // user properties
            if !data.properties.is_empty() {
                for (key, value) in data.properties.clone().iter() {
                    // if key has a space, replace by a _
                    let key = key.replace(" ", "_");
                    if let Some(value) = value.parse::<f64>().ok() {
                        user_property_number.insert(key, value);
                    } else {
                        user_property_string
                            .insert(key, value.clone());
                    }
                }
            }

            if user_property_string.len() > 0 {
                ga.user_property_string = Some(user_property_string);
            }
            if user_property_number.len() > 0 {
                ga.user_property_number = Some(user_property_number);
            }

            Ok(build_edgee_request(ga).map_err(|e| e.to_string())?)
        } else {
            Err("Missing user data".to_string())
        }
    }
}

fn build_edgee_request(ga: GaPayload) -> anyhow::Result<EdgeeRequest> {
    let mut headers = vec![];
    headers.push((String::from("content-length"), String::from("0")));

    let querystring = serde_qs::to_string(&ga)?;

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
                } else {
                    if chars[i + 2] == 'n' {
                        "upn"
                    } else {
                        "up"
                    }
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
}
