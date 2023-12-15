import asyncio
import requests


def main():
    res = requests.post("http://127.0.0.1:8484/partner/register",
                        json={
                            "country": "string",
                            "main_wallet": "string",
                            "name": "string",
                            "traffic_source": "string",
                            "users_amount_a_month": 0,
                            "login":"YeahNotSewerSide",
                            "password":"password"
                        })
    print(res.content)

    res = requests.post("http://127.0.0.1:8484/partner/login",
                        json={
                            "login":"YeahNotSewerSide",
                            "password":"password"
                        })
    
    print(res.content)

    res = requests.get("http://127.0.0.1:8484/partner/get",
                        headers={
                            "Authorization":"Bearer eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOm51bGwsInN1YiI6IlllYWhOb3RTZXdlclNpZGUiLCJleHAiOjEwMCwiaWF0IjoxMDAsImF1ZCI6IiJ9.5WZGk8qJFt0RBQG7yXxvNtIVjhXT1nrjeD7mkSMbRiY"
                        })
    
    print(res.content)


if __name__ == '__main__':
    main()
