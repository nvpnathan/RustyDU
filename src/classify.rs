use reqwest::{Client, header::{AUTHORIZATION, CONTENT_TYPE}};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

pub struct Classify {
    base_url: String,
    project_id: String,
    bearer_token: String,
}

#[derive(Serialize)]
struct ClassificationData<'a> {
    DocumentId: &'a str,
    #[serde(flatten)]
    prompts: Option<Value>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ClassificationResults {
    ClassificationResults: Vec<ClassificationResult>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ClassificationResult {
    DocumentTypeId: String,
    DocumentId: String,
    Confidence: f64,
    OcrConfidence: f64,
    Reference: Reference,
    DocumentBounds: DocumentBounds,
    ClassifierName: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Reference {
    TextStartIndex: usize,
    TextLength: usize,
    Tokens: Vec<Token>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Token {
    // Define token fields here
}

#[derive(Debug, serde::Deserialize)]
pub struct DocumentBounds {
    StartPage: usize,
    PageCount: usize,
    TextStartIndex: usize,
    TextLength: usize,
}

impl Classify {
    pub fn new(base_url: &str, project_id: &str, bearer_token: &str) -> Classify {
        Classify {
            base_url: base_url.to_string(),
            project_id: project_id.to_string(),
            bearer_token: bearer_token.to_string(),
        }
    }

    pub async fn classify_document(&self, document_id: &str, classifier: &str, prompts: Option<serde_json::Value>, validate_classification: bool) -> Option<String> {
        // Define the API endpoint for document classification
        let api_url = format!("{}/{}/classifiers/{}/classification?api-version=1", self.base_url, self.project_id, classifier);

        // Prepare request data
        let data = ClassificationData {
            DocumentId: document_id,
            prompts,
        };

        // Prepare request
        let client = Client::new();
        let response = client.post(&api_url)
            .header(AUTHORIZATION, format!("Bearer {}", self.bearer_token))
            .header(CONTENT_TYPE, "application/json")
            .json(&data)
            .send()
            .await;

        // Process response
        match response {
            Ok(response) => {
                match response.status() {
                    reqwest::StatusCode::OK => {
                        println!("Document successfully classified!");
                        let classification_result: ClassificationResult = response.json().await.unwrap();
                        if validate_classification {
                            Some(json!(classification_result).to_string())
                        } else {
                            let mut document_type_id = None;
                            let mut classification_confidence = None;
                            for result in classification_result.classificationResults {
                                if result.DocumentId == document_id {
                                    document_type_id = Some(result.DocumentTypeId);
                                    classification_confidence = Some(result.Confidence);
                                    break;
                                }
                            }
                            if let (Some(document_type_id), Some(classification_confidence)) = (document_type_id, classification_confidence) {
                                println!("Document Type ID: {}, Confidence: {}\n", document_type_id, classification_confidence);
                                Some(document_type_id)
                            } else {
                                println!("Document ID not found in classification results.");
                                None
                            }
                        }
                    },
                    _ => {
                        println!("Error: {} - {}", response.status(), response.text().await.unwrap());
                        None
                    }
                }
            },
            Err(e) => {
                println!("An error occurred during classification: {}", e);
                None
            }
        }
    }
}
