import requests
import json




url = "http://127.0.0.1:12345/jobs"
data = {
  "source_code": "fn main() { println!(\"Hello, world!\"); }",
  "language": "Rust",
  "user_id": 0,
  "contest_id": 0,
  "problem_id": 0
}

r = requests.post(url=url, json=data)
r = requests.get(url=url)
json_obj = json.loads(r.content)
print(json.dumps(json_obj, indent=2))

