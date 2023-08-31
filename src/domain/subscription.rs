use std::time::Duration;

use chrono::{DateTime, Days, Months, NaiveDate, Utc};
use iso_currency::Currency;
use uuid::Uuid;

use super::{ServiceId, UserId};

#[derive(Debug, serde::Serialize)]
pub(crate) struct Subscription {
    id: SubscriptionId,
    /// Owner of the subscription
    user_id: UserId,
    /// Name of subscription provided by user (or name of [`Service`](crate::domain::Service))
    name: String,
    /// Optional description of the subscription
    description: Option<String>,
    /// Amount in subunit for specified [`currency_code`] (e.g. cents for EUR)
    amount: u64,
    /// The currency code (ISO 4217 format) of the [`amount`](Subscription::amount)
    currency: Currency,
    /// Calculated next renewal date using [`billing_period`](Subscription::billing_period) and
    /// [`billing_period_unit`](Subscription::billing_period_unit)
    next_renewal_date: NaiveDate,
    /// Specifies that the subscription is reneved every X [`BillingPeriodUnit`](BillingPeriodUnit)
    billing_period: u8,
    /// Specifies how frequently is the subscription renewed. Used in combination with [`billing_period`](Subscription::billing_period)
    billing_period_unit: BillingPeriodUnit,
    /// ID of assigned service that this subscription is bound to
    service_id: ServiceId,
    /// Date when user added subscription to the system
    created_at: DateTime<Utc>,
    /// Last update date
    updated_at: DateTime<Utc>,
    /// Describes when the user marked subscription as cancelled
    cancelled_at: Option<DateTime<Utc>>,
    /// The reason for cancelling the subscription
    cancel_reason: Option<String>,
    /// Allows to set from when the user is subscribed to the service
    subscribed_at: Option<NaiveDate>,
    /// Date when user deleted a subscription
    deleted_at: Option<DateTime<Utc>>,
}

impl Subscription {
    pub fn next_renewal(&self) -> NaiveDate {
        match self.billing_period_unit {
            BillingPeriodUnit::Day => self
                .next_renewal_date
                .checked_add_days(Days::new(self.billing_period as u64 * 1))
                .expect("date out of range"),
            BillingPeriodUnit::Week => self
                .next_renewal_date
                .checked_add_days(Days::new(self.billing_period as u64 * 7))
                .expect("date out of range"),
            BillingPeriodUnit::Month => self
                .next_renewal_date
                .checked_add_months(Months::new(self.billing_period as u32 * 1))
                .expect("date out of range"),
            BillingPeriodUnit::Year => self
                .next_renewal_date
                .checked_add_months(Months::new(self.billing_period as u32 * 12))
                .expect("date out of range"),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct SubscriptionId(Uuid);

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum BillingPeriodUnit {
    Day,
    Week,
    Month,
    Year,
}
