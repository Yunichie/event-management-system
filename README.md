# Event Management System

A study case on Clean Architecture and Domain-Driven Design.

## Prerequisites

* Rust toolchain (1.75+)
* Docker

## Project Structure

### Root

```
event_ticketing/           
├── Cargo.toml             
├── Cargo.lock
├── Rocket.toml            
├── .env                   
├── docker-compose.yml     
├── README.md
├── migrations/            
│   ├── 20240001_create_enums.sql
│   ├── 20240002_create_events.sql
│   ├── 20240003_create_ticket_categories.sql
│   ├── 20240004_create_bookings.sql
│   ├── 20240005_create_tickets.sql
│   └── 20240006_create_refunds.sql
└── crates/
    ├── domain/
    ├── application/
    ├── infrastructure/
    └── api/
```

### Domain

```
crates/domain/src/
├── lib.rs
├── shared/
│   ├── mod.rs
│   ├── domain_event.rs     
│   └── errors.rs           
├── event/
│   ├── mod.rs
│   ├── aggregate.rs        
│   ├── ticket_category.rs  
│   ├── value_objects.rs    
│   ├── events.rs           
│   └── repository.rs       
├── booking/
│   ├── mod.rs
│   ├── aggregate.rs        
│   ├── value_objects.rs    
│   ├── events.rs
│   └── repository.rs       
├── ticket/
│   ├── mod.rs
│   ├── entity.rs           
│   ├── value_objects.rs    
│   ├── events.rs
│   └── repository.rs       
└── refund/
    ├── mod.rs
    ├── aggregate.rs        
    ├── value_objects.rs    
    ├── events.rs
    └── repository.rs       
```

### Application

```
crates/application/src/
├── lib.rs
├── errors.rs               
├── ports/
│   ├── mod.rs
│   ├── payment_gateway.rs  
│   ├── refund_service.rs   
│   └── notification.rs     
├── dto/
│   ├── mod.rs
│   ├── event_dto.rs
│   ├── booking_dto.rs
│   ├── ticket_dto.rs
│   └── refund_dto.rs
├── event/
│   ├── mod.rs
│   ├── commands.rs         
│   ├── handlers.rs         
│   ├── queries.rs          
│   └── query_handlers.rs
├── booking/
│   ├── mod.rs
│   ├── commands.rs         
│   ├── handlers.rs
│   ├── queries.rs          
│   └── query_handlers.rs
├── ticket/
│   ├── mod.rs
│   ├── commands.rs         
│   ├── handlers.rs
│   ├── queries.rs          
│   └── query_handlers.rs
└── refund/
    ├── mod.rs
    ├── commands.rs         
    ├── handlers.rs
    ├── queries.rs
    └── query_handlers.rs
```

### Infrastructure

```
crates/infrastructure/src/
├── lib.rs
├── persistence/
│   ├── mod.rs
│   ├── db.rs               
│   ├── models.rs           
│   ├── mappers.rs          
│   ├── event_repository.rs 
│   ├── booking_repository.rs
│   ├── ticket_repository.rs
│   └── refund_repository.rs
└── services/
    ├── mod.rs
    ├── payment_gateway.rs  
    ├── refund_service.rs   
    └── notification.rs     
```

### API

```
crates/api/src/
├── main.rs           
├── state.rs          
├── errors.rs         
├── fairings.rs       
├── catchers.rs       
├── guards/
│   ├── mod.rs
│   ├── auth_user.rs  
│   └── roles.rs      
├── request/          
│   ├── mod.rs
│   ├── event_requests.rs
│   ├── booking_requests.rs
│   └── refund_requests.rs
└── routes/
    ├── mod.rs        
    ├── events.rs
    ├── bookings.rs
    ├── tickets.rs
    └── refunds.rs
```

## Business Rules

### EVENT LIFECYCLE

**Status Transitions**
* Draft -> Published -> Cancelled
* Published -> Completed

**Business Rules**
* **BR1** A new event starts in Draft status.
* **BR2** End date must be after start date.
* **BR3** Maximum capacity must be greater than zero.
* **BR4** Event can only be published if it has at least one active ticket category.
* **BR5** Total ticket quota across all categories must not exceed maximum event capacity at publish time.
* **BR6** A Cancelled event cannot be published.
* **BR7** A Completed event cannot be cancelled.
* **BR8** When an event is cancelled, all paid bookings are marked as requiring a refund.

---

### TICKET CATEGORY

**Business Rules**
* **BR9** Ticket price cannot be negative.
* **BR10** Ticket quota must be greater than zero.
* **BR11** Sales end date must be on or before the event start date.
* **BR12** Sum of all category quotas must not exceed event maximum capacity.
* **BR13** A disabled category can still be stored for historical purposes.
* **BR14** Only active categories within their sales period and with remaining quota can be purchased.

---

### BOOKING LIFECYCLE

**Status Transitions**
* PendingPayment -> Paid -> Refunded
* PendingPayment -> Expired

**Business Rules**
* **BR15** Booking can only be created for a Published event with an active category within its sales period.
* **BR16** Ticket quantity must be > 0 and ≤ remaining quota of the selected category.
* **BR17** A customer cannot have more than one active booking per event.
* **BR18** Every booking has a payment deadline (e.g. 15 minutes after creation).
* **BR19** Payment amount must exactly equal the total booking price.
* **BR20** Booking cannot be paid after the payment deadline.
* **BR21** A Paid booking cannot be expired.
* **BR22** When a booking expires, the reserved quota is released back to the category.
* **BR23** Total price = (unit price × quantity) + optional service fee; never negative; represented as Money value object.
* **BR24** On payment, the system issues tickets with unique codes.

---

### TICKET & CHECK-IN

**Ticket Statuses**
* Active -> CheckedIn
* Active -> Cancelled

**Business Rules**
* **BR25** Check-in only allowed for an Active ticket matching the correct event, on the event day or within the allowed window.
* **BR26** A checked-in ticket cannot be checked in again.
* **BR27** On event cancellation, tickets become Cancelled or RefundRequired.

---

### REFUND LIFECYCLE

**Status Transitions**
* Requested -> Approved -> PaidOut
* Requested -> Rejected

**Business Rules**
* **BR28** Refund can only be requested for a Paid booking whose tickets have not been checked in, and before the refund deadline.
* **BR29** On event cancellation, refund is automatically allowed.
* **BR30** A refund can only be approved or rejected if its status is Requested.
* **BR31** Rejection requires a rejection reason.
* **BR32** On approval: tickets become Cancelled; booking becomes Refunded.
* **BR33** On rejection: booking stays Paid; Active tickets remain Active.
* **BR34** PaidOut requires a payment reference; a PaidOut refund cannot be changed again.

## Domain Model

### AGGREGATES & ENTITIES

#### **Event** `[Aggregate]`
* **id**: `EventId`
* **organizerId**: `UserId`
* **name**: `string`
* **description**: `string`
* **startDate**: `Date`
* **endDate**: `Date`
* **location**: `string`
* **maxCapacity**: `int`
* **status**: `EventStatus`
* **categories**: `TicketCategory[]`

#### **TicketCategory** `[Entity]`
* **id**: `CategoryId`
* **eventId**: `EventId`
* **name**: `string`
* **price**: `Money`
* **quota**: `int`
* **remainingQuota**: `int`
* **salesStart**: `Date`
* **salesEnd**: `Date`
* **isActive**: `bool`

#### **Booking** `[Aggregate]`
* **id**: `BookingId`
* **customerId**: `UserId`
* **eventId**: `EventId`
* **categoryId**: `CategoryId`
* **quantity**: `int`
* **totalPrice**: `Money`
* **status**: `BookingStatus`
* **paymentDeadline**: `DateTime`
* **tickets**: `Ticket[]`

#### **Ticket** `[Entity]`
* **id**: `TicketId`
* **bookingId**: `BookingId`
* **code**: `TicketCode`
* **status**: `TicketStatus`
* **checkedInAt**: `DateTime?`

#### **Refund** `[Aggregate]`
* **id**: `RefundId`
* **bookingId**: `BookingId`
* **customerId**: `UserId`
* **amount**: `Money`
* **status**: `RefundStatus`
* **reason**: `string?`
* **rejectionReason**: `string?`
* **paymentReference**: `string?`
* **requestedAt**: `DateTime`

---

### VALUE OBJECTS

#### **Money** `[Value Object]`
* **amount**: `Decimal`
* **currency**: `string`
* *Methods:* 
  * `add(Money) -> Money`
  * `multiply(int) -> Money`
  * `isNegative() -> bool`

#### **TicketCode** `[Value Object]`
* **value**: `string (UUID)`
* *Methods:*
  * `generate() -> TicketCode`
  * `equals(other) -> bool`

#### **EventId / BookingId** `[Value Object]`
* **value**: `UUID`
* *Methods:*
  * `equals(other) -> bool`

---

### DOMAIN EVENTS 
*(All raised by aggregate methods)*

* `EventCreated`
* `EventPublished`
* `EventCancelled`
* `TicketCategoryCreated`
* `TicketCategoryDisabled`
* `TicketReserved`
* `BookingPaid`
* `BookingExpired`
* `TicketCheckedIn`
* `RefundRequested`
* `RefundApproved`
* `RefundRejected`
* `RefundPaidOut`

---

### REPOSITORY INTERFACES

#### **IEventRepository** `[Repository]`
* `findById(id) -> Event?`
* `findPublished() -> Event[]`
* `save(event) -> void`

#### **IBookingRepository** `[Repository]`
* `findById(id) -> Booking?`
* `findByCustomerAndEvent() -> Booking?`
* `findPendingExpired() -> Booking[]`
* `save(booking) -> void`

#### **ITicketRepository** `[Repository]`
* `findByCode(code) -> Ticket?`
* `save(ticket) -> void`

#### **IRefundRepository** `[Repository]`
* `findById(id) -> Refund?`
* `findByBooking(bookingId) -> Refund?`
* `save(refund) -> void`


## Ubiquitous Languages

| Term | Meaning | Layer |
| :--- | :--- | :--- |
| **Event** | An activity organized by an Event Organizer that customers can attend via purchased tickets. | Aggregate |
| **Event Organizer** | A human actor who creates, publishes, and manages events; can approve or reject refunds. | Actor |
| **Customer** | A human actor who browses events, creates bookings, pays for tickets, and may request refunds. | Actor |
| **Gate Officer** | A human actor responsible for validating ticket codes and checking in participants at the venue. | Actor |
| **System Admin** | A human actor who triggers refund payouts and monitors operational processes. | Actor |
| **Ticket Category** | A named tier of ticket (e.g. Regular, VIP, Early Bird) with its own price, quota, and sales period, belonging to an Event. | Entity |
| **Quota** | The maximum number of tickets available in a specific Ticket Category. | Attribute |
| **Remaining Quota** | Quota minus the number of tickets currently reserved or sold; decremented on booking, incremented on expiry. | Derived attribute |
| **Sales Period** | The date range [salesStart, salesEnd] during which a Ticket Category can be purchased; salesEnd ≤ event start date. | Value Object |
| **Booking** | A temporary reservation of tickets made by a Customer before payment is completed; becomes a confirmed purchase once Paid. | Aggregate |
| **Payment Deadline** | The datetime by which a PendingPayment booking must be paid; after this the booking Expires and quota is released. | Attribute |
| **PendingPayment** | Booking status indicating payment has not been completed but the deadline has not passed. | Status |
| **Paid** | Booking status indicating payment was successful; tickets have been issued. | Status |
| **Expired** | Booking status indicating the payment deadline passed without successful payment; reserved quota is returned. | Status |
| **Refunded** | Booking status indicating an approved refund has been processed; tickets are cancelled. | Status |
| **Ticket** | Proof of attendance generated after a Booking is Paid; associated with a unique Ticket Code. | Entity |
| **Ticket Code** | An immutable, system-generated unique identifier on a Ticket used for validation at check-in. | Value Object |
| **Check-in** | The act of a Gate Officer validating a Ticket's code at the event venue, changing ticket status from Active to CheckedIn. | Domain action |
| **Money** | A value object pairing an immutable decimal Amount with a Currency; used for all price and fee calculations. | Value Object |
| **Refund** | The process and aggregate representing a customer's request to return money for a Paid booking. | Aggregate |
| **Refund Deadline** | The latest date/time at which a Customer may request a refund; after this, refunds are disallowed unless the event is cancelled. | Business rule |
| **Payment Gateway** | External system used to process booking payments; accessed only through the IPaymentGateway port. | Port |
| **Refund Payment Service** | External bank/payment service that disburses refund payouts to customers; accessed via IRefundPaymentService port. | Port |
| **Notification Service** | External system (email / WhatsApp) that sends transactional messages to users; accessed via INotificationService port. | Port |
| **Domain Event** | A record of something significant that happened within the domain (e.g. EventPublished, BookingPaid). Raised by aggregates; consumed by handlers. | Pattern |
| **Aggregate Root** | The entry point to a cluster of related domain objects (Event, Booking, Refund). All mutations go through the root. | Pattern |
| **Repository** | An abstraction over persistence for a single aggregate; defined as an interface in the domain layer, implemented in infrastructure. | Pattern |
| **Command** | An intent to change state (e.g. CreateEventCommand). Handled by a Command Handler in the application layer. | App layer |
| **Query** | A read-only request for data (e.g. GetAvailableEventsQuery). Handled by a Query Handler; does not change state. | App layer |
