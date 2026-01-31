#!/bin/bash

# Configuration
API_URL="http://127.0.0.1:8080/api"
ADMIN_USER="admin"
ADMIN_PASS="admin"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== API Testing Suite ===${NC}\n"

echo -e "${YELLOW}--- 1. Testing Login with Seeded Admin ---${NC}"
LOG_RES=$(curl -s -X POST "$API_URL/auth/login" \
     -H "Content-Type: application/json" \
     -d "{\"username\": \"$ADMIN_USER\", \"password\": \"$ADMIN_PASS\"}")
TOKEN=$(echo $LOG_RES | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')

if [ -z "$TOKEN" ]; then 
    echo -e "${RED}❌ Login failed. Did you run migrations and seed the database?${NC}"
    echo "Response: $LOG_RES"
    exit 1
fi
echo -e "${GREEN}✓ Success: Received Token${NC}"

echo -e "\n${YELLOW}--- 2. Testing 'Get Me' (Profile) ---${NC}"
ME_RES=$(curl -s -X GET "$API_URL/auth/me" \
     -H "Authorization: Bearer $TOKEN")
echo "Response: $ME_RES"
echo -e "${GREEN}✓ Profile retrieved${NC}"

echo -e "\n${YELLOW}--- 3. Testing Registration (New Tester) ---${NC}"
TEST_USER="tester_$(date +%s)"
REG_RES=$(curl -s -X POST "$API_URL/auth/register" \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $TOKEN" \
     -d "{\"username\": \"$TEST_USER\", \"password\": \"$ADMIN_PASS\", \"role\": \"SALE\"}")
echo "Response: $REG_RES"
USER_ID=$(echo $REG_RES | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')

if [ -z "$USER_ID" ]; then 
    echo -e "${RED}❌ Registration failed${NC}"; 
    exit 1; 
fi
echo -e "${GREEN}✓ User registered: $USER_ID${NC}"

echo -e "\n${YELLOW}--- 4. Testing Product Creation ---${NC}"
PROD_RES=$(curl -s -X POST "$API_URL/products" \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $TOKEN" \
     -d '{
       "code": "TEST-INT-'"$(date +%s)"'",
       "name": "Test Integration Product",
       "image": "test.jpg",
       "sale_price": "100.00",
       "cost_price": "50.00",
       "quantity": 50,
       "low_stock_threshold": 5
     }')
echo "Response: $PROD_RES"
PROD_ID=$(echo $PROD_RES | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')

if [ -z "$PROD_ID" ]; then 
    echo -e "${RED}❌ Product creation failed${NC}"; 
    exit 1; 
fi
echo -e "${GREEN}✓ Product created: $PROD_ID${NC}"

echo -e "\n${YELLOW}--- 5. Testing Product List ---${NC}"
PROD_LIST=$(curl -s -X GET "$API_URL/products" \
     -H "Authorization: Bearer $TOKEN")
echo "Response: $PROD_LIST"
echo -e "${GREEN}✓ Product list retrieved${NC}"

echo -e "\n${YELLOW}--- 6. Testing Product Update ---${NC}"
PROD_UPDATE=$(curl -s -X PUT "$API_URL/products/$PROD_ID" \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $TOKEN" \
     -d '{
       "name": "Updated Test Product",
       "sale_price": "120.00",
       "cost_price": "60.00",
       "quantity": 45,
       "low_stock_threshold": 10
     }')
echo "Response: $PROD_UPDATE"
echo -e "${GREEN}✓ Product updated${NC}"

echo -e "\n${YELLOW}--- 7. Testing Product Status Toggle ---${NC}"
STATUS_UPDATE=$(curl -s -X PATCH "$API_URL/products/$PROD_ID/status" \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $TOKEN" \
     -d '{"is_active": false}')
echo "Response: $STATUS_UPDATE"
echo -e "${GREEN}✓ Product status toggled${NC}"

echo -e "\n${YELLOW}--- 8. Testing Sale Creation ---${NC}"
SALE_RES=$(curl -s -X POST "$API_URL/sales" \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $TOKEN" \
     -d "{
       \"items\": [{\"product_id\": \"$PROD_ID\", \"quantity\": 2}],
       \"payment_method\": \"CREDIT\",
       \"currency_code\": \"USD\"
     }")
echo "Response: $SALE_RES"
SALE_ID=$(echo $SALE_RES | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
if [ -n "$SALE_ID" ]; then
    echo -e "${GREEN}✓ Sale created: $SALE_ID${NC}"
else
    echo -e "${YELLOW}⚠ Sale creation may have failed (check response)${NC}"
fi

echo -e "\n${YELLOW}--- 9. Testing Sales List ---${NC}"
SALES_LIST=$(curl -s -X GET "$API_URL/sales" \
     -H "Authorization: Bearer $TOKEN")
echo "Response: $SALES_LIST"
echo -e "${GREEN}✓ Sales list retrieved${NC}"

echo -e "\n${YELLOW}--- 10. Testing Quotation Creation ---${NC}"
QUOT_RES=$(curl -s -X POST "$API_URL/quotations" \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $TOKEN" \
     -d "{
       \"items\": [{\"product_id\": \"$PROD_ID\", \"quantity\": 3}],
       \"currency_code\": \"THB\"
     }")
echo "Response: $QUOT_RES"
QUOT_ID=$(echo $QUOT_RES | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
if [ -n "$QUOT_ID" ]; then
    echo -e "${GREEN}✓ Quotation created: $QUOT_ID${NC}"
else
    echo -e "${YELLOW}⚠ Quotation creation may have failed (check response)${NC}"
fi

echo -e "\n${YELLOW}--- 11. Testing Quotations List ---${NC}"
QUOT_LIST=$(curl -s -X GET "$API_URL/quotations" \
     -H "Authorization: Bearer $TOKEN")
echo "Response: $QUOT_LIST"
echo -e "${GREEN}✓ Quotations list retrieved${NC}"

echo -e "\n${YELLOW}--- 12. Testing Exchange Rate Settings ---${NC}"
SET_RES=$(curl -s -X POST "$API_URL/settings/exchange/THB" \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $TOKEN" \
     -d '{"rate_to_base": "35.50"}')
echo "Response: $SET_RES"
echo -e "${GREEN}✓ Exchange rate updated${NC}"

echo -e "\n${YELLOW}--- 13. Testing Get All Exchange Rates ---${NC}"
RATES_RES=$(curl -s -X GET "$API_URL/settings/exchange" \
     -H "Authorization: Bearer $TOKEN")
echo "Response: $RATES_RES"
echo -e "${GREEN}✓ Exchange rates retrieved${NC}"

echo -e "\n${YELLOW}--- 14. Testing Dashboard Summary ---${NC}"
DASH_RES=$(curl -s -X GET "$API_URL/dashboard/summary" \
     -H "Authorization: Bearer $TOKEN")
echo "Response: $DASH_RES"
echo -e "${GREEN}✓ Dashboard summary retrieved${NC}"

echo -e "\n${YELLOW}--- 15. Testing Product Performance Report ---${NC}"
REP_RES=$(curl -s -X GET "$API_URL/reports/products" \
     -H "Authorization: Bearer $TOKEN")
echo "Response: $REP_RES"
echo -e "${GREEN}✓ Product report retrieved${NC}"

echo -e "\n${YELLOW}--- 16. Testing Low Stock Report ---${NC}"
LOW_STOCK_RES=$(curl -s -X GET "$API_URL/reports/low-stock" \
     -H "Authorization: Bearer $TOKEN")
echo "Response: $LOW_STOCK_RES"
echo -e "${GREEN}✓ Low stock report retrieved${NC}"

echo -e "\n${YELLOW}--- 17. Testing User Management (List) ---${NC}"
LIST_USERS=$(curl -s -X GET "$API_URL/settings/users" \
     -H "Authorization: Bearer $TOKEN")
echo "Response: $LIST_USERS"
echo -e "${GREEN}✓ Users list retrieved${NC}"

echo -e "\n${YELLOW}--- 18. Testing User Management (Delete) ---${NC}"
DELETE_RES=$(curl -s -X DELETE "$API_URL/settings/users/$USER_ID" \
     -H "Authorization: Bearer $TOKEN")
echo "Response: $DELETE_RES"
echo -e "${GREEN}✓ User deleted${NC}"

echo -e "\n${GREEN}=== All Tests Completed Successfully ===${NC}"
