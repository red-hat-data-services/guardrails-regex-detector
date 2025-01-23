use std::{collections::HashMap, error::Error};

use regex::Regex;

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
struct DetectionResponse {
    start: usize,
    end: usize,
    text: String,
    detection_type: String,
    detection: String,
    score: f64,
}

#[derive(Debug, Deserialize)]
pub struct DetectionRequest {
    contents: Vec<String>,
    detector_params: DetectorParams,
}

#[derive(Debug, Deserialize)]
pub struct DetectorParams {
    regex: Vec<String>,
}

struct RegexDetection {
    regex: String,
    detection_type: String,
    detection: String,
}

fn email_address_detector(content: &String) -> Result<Vec<DetectionResponse>, Box<dyn Error>> {
    let regex = r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}".to_string();

    let email_detection = RegexDetection {
        regex,
        detection_type: String::from("pii"),
        detection: String::from("EmailAddress"),
    };

    regex_match(email_detection, content)
}

fn ssn_detector(content: &String) -> Result<Vec<DetectionResponse>, Box<dyn Error>> {
    let regex = r"\b(?:([0-9]{5})-([0-9]{4})|([0-9]{3})-([0-9]{6})|(([0-9]{3})-([0-9]{2})-([0-9]{4}))|[0-9]{9}|([0-9]{3})[- .]([0-9]{2})[- .]([0-9]{4}))\b".to_string();

    let ssn_detection = RegexDetection {
        regex,
        detection_type: String::from("pii"),
        detection: String::from("SocialSecurity"),
    };

    regex_match(ssn_detection, content)
}

fn credit_card_detector(content: &String) -> Result<Vec<DetectionResponse>, Box<dyn Error>> {
    let regex = r"\b((4\d{3})|(5[0-5]\d{2})|(6\d{3})|(1\d{3})|(3\d{3}))[- ]?(\d{3,4})[- ]?(\d{3,4})[- ]?(\d{3,5})\b".to_string();

    let cc_detection = RegexDetection {
        regex,
        detection_type: String::from("pii"),
        detection: String::from("CreditCard"),
    };

    regex_match(cc_detection, content)
}

fn regex_match(
    regex_detection: RegexDetection,
    content: &str,
) -> Result<Vec<DetectionResponse>, Box<dyn Error>> {
    if let Ok(re) = Regex::new(&regex_detection.regex) {
        let mut detections: Vec<DetectionResponse> = Vec::new();
        for matched in re.find_iter(&content) {
            detections.push(DetectionResponse {
                start: matched.start(),
                end: matched.end(),
                detection_type: regex_detection.detection_type.clone(),
                text: matched.as_str().to_owned(),
                detection: regex_detection.detection.clone(),
                score: 1.0,
            });
        }
        return Ok(detections);
    } else {
        return Err(format!("invalid regex pattern: {}", regex_detection.regex).into());
    }
}

// pub async fn handle_text_contents(Json(payload): Json<DetectionRequest>) {
//     info!("incoming payload: {:#?}", payload);
// }
pub async fn handle_text_contents(
    Json(payload): Json<DetectionRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    println!("hi");
    info!("incoming payload: {:?}", payload);
    if payload.detector_params.regex.is_empty() {
        return Err((StatusCode::BAD_REQUEST, format!("empty regex")));
    }

    let mut detections: Vec<DetectionResponse> = Vec::new();
    let mut builtin_regex: HashMap<
        &str,
        fn(&String) -> Result<Vec<DetectionResponse>, Box<dyn Error>>,
    > = HashMap::new();

    builtin_regex.insert("email", email_address_detector);
    builtin_regex.insert("ssn", ssn_detector);
    builtin_regex.insert("credit-card", credit_card_detector);

    for content in &payload.contents {
        for re in &payload.detector_params.regex {
            if let Some(detector) = builtin_regex.get(re.as_str()) {
                if let Ok(mut results) = detector(content) {
                    detections.append(&mut results);
                };
            } else {
                let detection = RegexDetection {
                    regex: re.to_string(),
                    detection_type: "custom".to_string(),
                    detection: "custom".to_string(),
                };
                if let Ok(mut results) = regex_match(detection, content) {
                    detections.append(&mut results);
                } else {
                    tracing::warn!("could not process regex: {re}")
                };
            }
        }
    }

    Ok(Json(json!([detections])).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // regex matches inputs with numbers only
    fn test_regex_match() {
        let detection = RegexDetection {
            regex: r"^[0-9]+$".to_string(),
            detection_type: "number".to_string(),
            detection: "number".to_string(),
        };

        let content = "123456";

        let expected = vec![DetectionResponse {
            start: 0,
            end: 6,
            text: "123456".to_string(),
            detection_type: "number".to_string(),
            detection: "number".to_string(),
            score: 1.0,
        }];

        let received = regex_match(detection, content).unwrap();
        assert_eq!(received, expected);
    }
}
