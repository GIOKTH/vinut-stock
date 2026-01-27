#!/bin/bash

# Configuration
API_URL="http://127.0.0.1:8080/api"
ADMIN_USER="admin"
ADMIN_PASS="admin"

echo -e "\n--- 1. Testing Login with Seeded Admin ---"
LOG_RES=$(curl -s -X POST "$API_URL/auth/login" \
     -H "Content-Type: application/json" \
     -d "{\"username\": \"$ADMIN_USER\", \"password\": \"$ADMIN_PASS\"}")
TOKEN=$(echo $LOG_RES | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')

if [ -z "$TOKEN" ]; then 
    echo "Login failed. Did you run migrations and seed the database?"
    echo "Response: $LOG_RES"
    exit 1
fi
echo "Success: Received Token"

echo -e "\n--- 1.5 Testing 'Get Me' (Profile) ---"
ME_RES=$(curl -s -X GET "$API_URL/auth/me" \
     -H "Authorization: Bearer $TOKEN")
echo "Get Me Response: $ME_RES"

echo -e "\n--- 2. Testing Registration (New Tester) ---"
TEST_USER="tester_$(date +%s)"
REG_RES=$(curl -s -X POST "$API_URL/auth/register" \
     -H "Content-Type: application/json" \
     -d "{\"username\": \"$TEST_USER\", \"password\": \"$ADMIN_PASS\", \"role\": \"SALE\"}")
echo "Registration Response: $REG_RES"
USER_ID=$(echo $REG_RES | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')

if [ -z "$USER_ID" ]; then echo "Registration failed"; exit 1; fi

echo -e "\n--- 3. Testing Product Creation ---"
PROD_RES=$(curl -s -X POST "$API_URL/products" \
     -H "Content-Type: application/json" \
     -d '{
       "code": "TEST-INT-'"$(date +%s)"'",
       "name": "Test Integration Product",
       "image": "test.jpg",
       "sale_price": "100.00",
       "quantity": 50,
       "low_stock_threshold": 5
     }')
echo "Product Response: $PROD_RES"
PROD_ID=$(echo $PROD_RES | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')

if [ -z "$PROD_ID" ]; then echo "Product creation failed"; exit 1; fi

echo -e "\n--- 4. Testing Sale Creation ---"
SALE_RES=$(curl -s -X POST "$API_URL/sales" \
     -H "Content-Type: application/json" \
     -d "{
       \"items\": [{\"product_id\": \"$PROD_ID\", \"quantity\": 1}],
       \"payment_method\": \"CREDIT\"
     }")
echo "Sale Response: $SALE_RES"

echo -e "\n--- 5. Testing Settings (Exchange Rate) ---"
SET_RES=$(curl -s -X POST "$API_URL/settings/exchange/THB" \
     -H "Content-Type: application/json" \
     -d '{"rate_to_base": "35.50"}')
echo "Settings Update Response: $SET_RES"

echo -e "\n--- 6. Testing Dashboard & Reports ---"
DASH_RES=$(curl -s -X GET "$API_URL/dashboard/summary")
echo "Dashboard Summary: $DASH_RES"

REP_RES=$(curl -s -X GET "$API_URL/reports/products")
echo "Product Report: $REP_RES"

echo -e "\n--- 7. Testing User Management (List) ---"
LIST_USERS=$(curl -s -X GET "$API_URL/settings/users")
echo "Users List: $LIST_USERS"

echo -e "\n--- 8. Testing User Management (Delete) ---"
DELETE_RES=$(curl -s -X DELETE "$API_URL/settings/users/$USER_ID")
echo "Delete User Response: $DELETE_RES"

echo -e "\n--- All Tests Completed Successfully ---"
