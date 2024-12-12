use exports::provider::{Data, Dict, EdgeeRequest, Event, Guest};
use ga_payload::{GaPayload, Product};
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
                if let Some(value) = value.parse::<f64>().ok() {
                    event_parameter_number.insert(key, value);
                } else {
                    if key == "currency" {
                        ga.currency_code = Some(value.clone());
                    } else {
                        event_parameter_string.insert(key, value.clone());
                    }
                }
            }

            ga.event_parameter_string = Some(event_parameter_string);
            if event_parameter_number.len() > 0 {
                ga.event_parameter_number = Some(event_parameter_number);
            }

            Ok(build_edgee_request(ga, vec![]).map_err(|e| e.to_string())?)
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
            event_parameter_string.insert("event_id".to_string(), edgee_event.uuid.clone());

            let mut event_parameter_number = HashMap::new();

            for (key, value) in data.properties.iter() {
                let key = key.replace(" ", "_");
                if let Some(value) = value.parse::<f64>().ok() {
                    event_parameter_number.insert(key, value);
                } else {
                    if key == "currency" {
                        ga.currency_code = Some(value.clone());
                    } else {
                        event_parameter_string.insert(key, value.clone());
                    }
                }
            }

            ga.event_parameter_string = Some(event_parameter_string);
            if event_parameter_number.len() > 0 {
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

    fn user(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if let Data::User(ref data) = edgee_event.data {
            if data.user_id.is_empty() && data.anonymous_id.is_empty() {
                return Err("user_id or anonymous_id is not set".to_string());
            }

            let mut ga = GaPayload::new(&edgee_event, cred_map, "user".to_string())
                .map_err(|e| e.to_string())?;

            let mut event_parameter_string = HashMap::new();
            event_parameter_string.insert("event_id".to_string(), edgee_event.uuid.clone());
            ga.event_parameter_string = Some(event_parameter_string);

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
                        user_property_string.insert(key, value.clone());
                    }
                }
            }

            if user_property_string.len() > 0 {
                ga.user_property_string = Some(user_property_string);
            }
            if user_property_number.len() > 0 {
                ga.user_property_number = Some(user_property_number);
            }

            Ok(build_edgee_request(ga, vec![]).map_err(|e| e.to_string())?)
        } else {
            Err("Missing user data".to_string())
        }
    }
}

fn build_edgee_request(ga: GaPayload, ga_items: Vec<Product>) -> anyhow::Result<EdgeeRequest> {
    let mut headers = vec![];
    headers.push((String::from("content-length"), String::from("0")));

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
