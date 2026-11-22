-- 店名和用户名一样：只在「待审」之间互斥，被拒后可以再申请
CREATE UNIQUE INDEX uq_pending_shop_name
    ON merchant_applications (shop_name) WHERE status = 'pending';
