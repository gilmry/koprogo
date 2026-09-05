ALTER TABLE call_for_funds DROP CONSTRAINT IF EXISTS chk_call_for_funds_reserve_share;
ALTER TABLE call_for_funds DROP COLUMN IF EXISTS reserve_fund_share;
