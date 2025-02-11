# Regex Detector

A lightweight HTTP server designed to parse text using predefined patterns or custom regular expressions.It serves as a detection service, primarily integrated with the [FMS Guardrails Orchestrator](https://github.com/foundation-model-stack/fms-guardrails-orchestrator).

### Registered patterns
- ssn (social security number)
- credit-card
- email

### Sample request
```bash
curl -X POST http://localhost:8080/api/v1/text/contents \
  -H "Content-Type: application/json" \
  -d '{
    "contents": [
      "hello", 
      "this is my email address email@domain.com", 
      "check out my social 123-45-6789", 
      "this text should not pop up", 
      "my amex 374245455400126"
    ], 
    "detector_params": {
      "regex": [
        "ssn", 
        "email", 
        "credit-card", 
        "^hello$"
      ]
    }
  }'
```

### Sample response
```bash
[
  [
    {
      "detection": "custom",
      "detection_type": "custom",
      "end": 2,
      "score": 1.0,
      "start": 0,
      "text": "hi"
    },
    {
      "detection": "EmailAddress",
      "detection_type": "pii",
      "end": 41,
      "score": 1.0,
      "start": 25,
      "text": "email@domain.com"
    },
    {
      "detection": "SocialSecurity",
      "detection_type": "pii",
      "end": 31,
      "score": 1.0,
      "start": 20,
      "text": "123-45-6789"
    },
    {
      "detection": "CreditCard",
      "detection_type": "pii",
      "end": 23,
      "score": 1.0,
      "start": 8,
      "text": "374245455400126"
    }
  ]
]
```
