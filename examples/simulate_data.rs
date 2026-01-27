use dotenv::dotenv;
use rand::Rng;
use rust_decimal::Decimal;
use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("--- Seeding 100 Products ---");
    let mut product_ids = Vec::new();
    let mut rng = rand::thread_rng();

    for i in 1..=100 {
        let id = Uuid::new_v4();
        let code = format!("FIX-PROD-{:03}-{:04}", i, rng.gen_range(1000..9999));
        let name = format!("Pro Item {}", i);

        let val = rng.gen_range(1000..30000);
        let sale_price = Decimal::new(val, 2);

        let cost_percent = rng.gen_range(50..70); // 50-70% cost to ensure healthy profit
        let cost_price = (sale_price * Decimal::from(cost_percent)) / Decimal::from(100);

        let quantity = rng.gen_range(50..500);
        let threshold = 10;

        sqlx::query!(
            "INSERT INTO products (id, code, name, sale_price, cost_price, quantity, low_stock_threshold) 
             VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (code) DO NOTHING",
            id, code, name, sale_price, cost_price, quantity, threshold
        )
        .execute(&pool)
        .await?;

        let prod_id = sqlx::query!("SELECT id FROM products WHERE code = $1", code)
            .fetch_one(&pool)
            .await?
            .id;
        product_ids.push(prod_id);
    }
    println!("Successfully seeded products.");

    println!("--- Simulating 100 Multi-Currency Sales (Corrected) ---");

    let user_result = sqlx::query!("SELECT id FROM users LIMIT 1")
        .fetch_one(&pool)
        .await?;
    let user_id = user_result.id;

    let currencies = vec!["BASE", "THB", "LAK", "USD", "CNY", "AUD"];

    for i in 1..=100 {
        let sale_id = Uuid::new_v4();
        let payment_method = if rng.gen_bool(0.7) {
            "CASH"
        } else {
            "TRANSFER"
        };

        let c_idx = rng.gen_range(0..currencies.len());
        let currency_code = currencies[c_idx];

        let exchange_rate = if currency_code == "BASE" {
            Decimal::new(1, 0)
        } else {
            sqlx::query!(
                "SELECT rate_to_base FROM exchange_rates WHERE currency_code = $1",
                currency_code
            )
            .fetch_one(&pool)
            .await?
            .rate_to_base
        };

        let mut items_to_insert = Vec::new();
        let num_items = rng.gen_range(1..4);
        let mut total_sale_amount_base = Decimal::new(0, 2);

        for _ in 0..num_items {
            let p_idx = rng.gen_range(0..product_ids.len());
            let product_id = product_ids[p_idx];
            let product = sqlx::query!("SELECT sale_price FROM products WHERE id = $1", product_id)
                .fetch_one(&pool)
                .await?;

            let qty = rng.gen_range(1..5);

            // CONVERT Sale Price from Base to Local currency
            let unit_price_local = product.sale_price * exchange_rate;
            let subtotal_local = unit_price_local * Decimal::from(qty);

            total_sale_amount_base += (product.sale_price * Decimal::from(qty));
            items_to_insert.push((product_id, qty, unit_price_local, subtotal_local));
        }

        let total_sale_amount_local = total_sale_amount_base * exchange_rate;

        sqlx::query!(
            "INSERT INTO sales (id, user_id, total_amount, payment_method, currency_code, exchange_rate) 
             VALUES ($1, $2, $3, $4, $5, $6)",
            sale_id, user_id, total_sale_amount_local, payment_method, currency_code, exchange_rate
        )
        .execute(&pool)
        .await?;

        for (pid, qty, price, sub) in items_to_insert {
            sqlx::query!(
                "INSERT INTO sale_items (id, sale_id, product_id, quantity, unit_price, subtotal) 
                 VALUES ($1, $2, $3, $4, $5, $6)",
                Uuid::new_v4(),
                sale_id,
                pid,
                qty,
                price,
                sub
            )
            .execute(&pool)
            .await?;
        }

        if i % 10 == 0 {
            println!("Simulated {} sales in {}...", i, currency_code);
        }
    }
    println!("Sales Simulation complete.");

    println!("--- Simulating 50 Multi-Currency Batch Purchases (Restocking) ---");
    let suppliers = vec![
        Uuid::new_v4(), // Random supplier IDs for simulation
        Uuid::new_v4(),
        Uuid::new_v4(),
    ];

    // Seed suppliers if needed
    for s_id in &suppliers {
        sqlx::query!(
            "INSERT INTO suppliers (id, name, email) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING",
            s_id,
            format!("Supplier-{}", rng.gen_range(100..999)),
            "contact@supplier.com"
        )
        .execute(&pool)
        .await
        .ok();
    }

    for i in 1..=50 {
        let purchase_id = Uuid::new_v4();
        let s_idx = rng.gen_range(0..suppliers.len());
        let supplier_id = suppliers[s_idx];

        let c_idx = rng.gen_range(0..currencies.len());
        let currency_code = currencies[c_idx];

        let exchange_rate = if currency_code == "BASE" {
            Decimal::new(1, 0)
        } else {
            sqlx::query!(
                "SELECT rate_to_base FROM exchange_rates WHERE currency_code = $1",
                currency_code
            )
            .fetch_one(&pool)
            .await?
            .rate_to_base
        };

        // Create Purchase Items first to calculate total
        let num_items = rng.gen_range(1..5);
        let mut items_to_insert = Vec::new();
        let mut total_amount_foreign = Decimal::new(0, 2);

        for _ in 0..num_items {
            let p_idx = rng.gen_range(0..product_ids.len());
            let product_id = product_ids[p_idx];
            let qty = rng.gen_range(50..200);

            // Buy price usually lower than sale price
            let product = sqlx::query!("SELECT cost_price FROM products WHERE id = $1", product_id)
                .fetch_one(&pool)
                .await?;

            let base_cost = product.cost_price.unwrap_or(Decimal::new(0, 0));
            let variation = Decimal::from(rng.gen_range(90..110)) / Decimal::from(100);
            let buy_price_base = base_cost * variation;
            let buy_price_foreign = if exchange_rate.is_zero() {
                Decimal::new(0, 0)
            } else {
                buy_price_base / exchange_rate
            };
            let subtotal_foreign = buy_price_foreign * Decimal::from(qty);

            total_amount_foreign += subtotal_foreign;
            items_to_insert.push((product_id, qty, buy_price_foreign, subtotal_foreign));
        }

        sqlx::query!(
            "INSERT INTO purchases (id, supplier_id, total_amount, currency_code, exchange_rate, created_at) 
             VALUES ($1, $2, $3, $4, $5, NOW() - (random() * (INTERVAL '90 days')))",
            purchase_id, supplier_id, total_amount_foreign, currency_code, exchange_rate
        )
        .execute(&pool)
        .await?;

        for (pid, qty, price, sub) in items_to_insert {
            sqlx::query!(
                "INSERT INTO purchase_items (id, purchase_id, product_id, quantity, buy_price, subtotal) 
                 VALUES ($1, $2, $3, $4, $5, $6)",
                Uuid::new_v4(),
                purchase_id,
                pid,
                qty,
                price,
                sub
            )
            .execute(&pool)
            .await?;

            sqlx::query!(
                "UPDATE products SET quantity = quantity + $1 WHERE id = $2",
                qty,
                pid
            )
            .execute(&pool)
            .await?;
        }

        if i % 10 == 0 {
            println!("Simulated {} batch purchases...", i);
        }
    }
    println!("Purchase Simulation complete.");

    Ok(())
}
