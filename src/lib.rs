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
