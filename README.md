# Recurio

## Idea

> A tracker of recurring payments/subscriptions? Nowadays almost every service wants you to pay in a subscription model. It is super easy to forger that you're actually paying for something you don't use. The user would create a record for what they're paying and what amount. They could get some kind of notification about upcoming payment or option to remind them to cancel subscription on specific date.
> It could track the the price change, create some stats of how much money the user spends on recurring subscriptions, maybe even option to mark subscription as split between flatmates?

## Name

"Recurio" is a fusion of "recurring" and "curio" (short for "curiosity"). This name implies a service that explores and manages recurring payments with curiosity, helping users stay informed about their financial commitments.

## Features

- [ ] Track recurring payments/subscriptions
- [ ] Notifications for upcoming payments (separate setting for each subscription)
- [ ] Reminder to cancel a subscription on a specific date
- [ ] Reminder to cancel a subscription after specified billing cycles
- [ ] Archive/Cancel old subscriptions
  - [ ] Reason for cancellation (optional)
- [ ] Filter only active subscriptions
- [ ] Statistics on subscription expenses
- [ ] Option to mark subscription as split (only fraction of costs taken into consideration) - each subscription has a `share` value (0 - 100)
- [ ] SMS notifications
- [ ] Email notifications
- [ ] Guest users (no registration required, 30 days of data storage)
- [ ] Rate limiting
- [ ] Payment history (with option to edit/delete automatically created payments)
  - [ ] (Optional) Audit trail (e.g. "User changed payment amount from $9.99 to $7.99 on 2024-05-01")
  - [ ] Payments can be edited only for a specific time period (e.g. a month)
  - [ ] Option to add past payments (date range + price, date range + price)
- [ ] Preferred currency by the user + exchange rates
