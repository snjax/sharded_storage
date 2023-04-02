import json
import requests

data = ['1', '2', '3']

json_data = json.dumps(data)
headers = {'Content-Type': 'application/json'}

response = requests.post('http://localhost:3000/data', data=json_data, headers=headers)
print('POST /data result:', response)

for i in range(4):
    response = requests.get(f'http://localhost:300{i}/data/partial')
    print(f'Partial data for peer {i}:', response.json())

response = requests.get('http://localhost:3000/data')
if response.status_code == 200:
    print('Whole data:', response.json())
else:
    print('Could not get whole data:', response)
