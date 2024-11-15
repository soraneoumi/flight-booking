use std::collections::HashMap;
use std::io::{self, BufRead};
use chrono::{NaiveDateTime, Duration};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SeatType {
    A,
    B,
    C,
    D,
}

impl SeatType {
    fn as_char(&self) -> char {
        match self {
            SeatType::A => 'A',
            SeatType::B => 'B',
            SeatType::C => 'C',
            SeatType::D => 'D',
        }
    }

    fn from_char(c: char) -> Option<Self> {
        match c {
            'A' => Some(SeatType::A),
            'B' => Some(SeatType::B),
            'C' => Some(SeatType::C),
            'D' => Some(SeatType::D),
            _ => None,
        }
    }

    fn variants() -> [SeatType; 4] {
        [SeatType::A, SeatType::B, SeatType::C, SeatType::D]
    }
}

#[derive(Clone)]
struct SeatClass {
    column: u32,
    price: u32,
}

#[derive(Clone)]
struct Flight {
    flight_id: u32,
    departure_airport: u32,
    arrival_airport: u32,
    departure_time: String,
    arrival_time: String,
    seat_classes: Vec<SeatClass>,
}

impl Flight {
    fn new(
        flight_id: u32,
        departure_airport: u32,
        arrival_airport: u32,
        departure_time: String,
        arrival_time: String,
        seat_classes: Vec<SeatClass>,
    ) -> Self {
        Flight {
            flight_id,
            departure_airport,
            arrival_airport,
            departure_time,
            arrival_time,
            seat_classes,
        }
    }

    fn get_seat_class(&self, seat_id: &str) -> Option<(u32, u32)> {
        let (row_part, seat_type_char) = seat_id.split_at(seat_id.len() - 1);
        let row: u32 = row_part.parse().ok()?;
        let _seat_type = SeatType::from_char(seat_type_char.chars().next()?)?;

        for (i, seat_class) in self.seat_classes.iter().enumerate() {
            if row <= seat_class.column {
                return Some((i as u32 + 1, seat_class.price));
            }
        }
        None
    }
}

struct Reservation {
    reservation_id: u32,
    user_id: String,
    date: String,
    flight_id: u32,
    seat_id: String,
    price: u32,
    is_cancelled: bool,
}

impl Reservation {
    fn new(
        reservation_id: u32,
        user_id: String,
        date: String,
        flight_id: u32,
        seat_id: String,
        price: u32,
    ) -> Self {
        Reservation {
            reservation_id,
            user_id,
            date,
            flight_id,
            seat_id,
            price,
            is_cancelled: false,
        }
    }
}

struct ReservationSystem {
    flights: HashMap<u32, Flight>,
    reservations: HashMap<u32, Reservation>,
    seat_reservations: HashMap<String, HashMap<u32, HashMap<String, bool>>>,
    next_reservation_id: u32,
}

impl ReservationSystem {
    fn new() -> Self {
        ReservationSystem {
            flights: HashMap::new(),
            reservations: HashMap::new(),
            seat_reservations: HashMap::new(),
            next_reservation_id: 1,
        }
    }

    fn add_flight(
        &mut self,
        flight_id: u32,
        departure_airport: u32,
        arrival_airport: u32,
        departure_time: String,
        arrival_time: String,
        seat_classes: Vec<SeatClass>,
    ) {
        let flight = Flight::new(
            flight_id,
            departure_airport,
            arrival_airport,
            departure_time,
            arrival_time,
            seat_classes,
        );
        self.flights.insert(flight_id, flight);
    }

    fn parse_datetime(&self, date: &str, time: &str) -> Option<NaiveDateTime> {
        let datetime_str = format!("{}-{}", date, time);
        NaiveDateTime::parse_from_str(&datetime_str, "%Y/%m/%d-%H:%M:%S").ok()
    }

    fn is_too_late(&self, current_datetime: NaiveDateTime, flight_datetime: NaiveDateTime) -> bool {
        current_datetime >= flight_datetime - Duration::hours(2)
    }

    fn get_flight_datetime(&self, date: &str, flight: &Flight) -> Option<NaiveDateTime> {
        self.parse_datetime(date, &flight.departure_time)
    }

    fn is_seat_reserved(&self, date: &str, flight_id: u32, seat_id: &str) -> bool {
        if let Some(flights_on_date) = self.seat_reservations.get(date) {
            if let Some(seats) = flights_on_date.get(&flight_id) {
                if let Some(&reserved) = seats.get(seat_id) {
                    return reserved;
                }
            }
        }
        false
    }

    fn reserve_seat(&mut self, date: &str, flight_id: u32, seat_id: &str) {
        self.seat_reservations
            .entry(date.to_string())
            .or_insert_with(HashMap::new)
            .entry(flight_id)
            .or_insert_with(HashMap::new)
            .insert(seat_id.to_string(), true);
    }

    fn unreserve_seat(&mut self, date: &str, flight_id: u32, seat_id: &str) {
        if let Some(flights_on_date) = self.seat_reservations.get_mut(date) {
            if let Some(seats) = flights_on_date.get_mut(&flight_id) {
                seats.insert(seat_id.to_string(), false);
            }
        }
    }

    fn process_reserve(
        &mut self,
        current_datetime: &str,
        user_id: &str,
        date: &str,
        flight_id: u32,
        seat_id: &str,
    ) -> String {
        if !self.flights.contains_key(&flight_id) {
            return "reserve: flight not found".to_string();
        }

        let flight = self.flights.get(&flight_id).unwrap();
        let current_dt = match NaiveDateTime::parse_from_str(current_datetime, "%Y/%m/%d-%H:%M:%S") {
            Ok(dt) => dt,
            Err(_) => return "reserve: invalid datetime".to_string(),
        };

        let flight_dt = match self.get_flight_datetime(date, flight) {
            Some(dt) => dt,
            None => return "reserve: invalid flight datetime".to_string(),
        };

        if self.is_too_late(current_dt, flight_dt) {
            return "reserve: too late".to_string();
        }

        if self.is_seat_reserved(date, flight_id, seat_id) {
            return "reserve: already reserved".to_string();
        }

        let (_, price) = match flight.get_seat_class(seat_id) {
            Some((sc, pr)) => (sc, pr),
            None => return "reserve: invalid seat_id".to_string(),
        };

        let reservation = Reservation::new(
            self.next_reservation_id,
            user_id.to_string(),
            date.to_string(),
            flight_id,
            seat_id.to_string(),
            price,
        );
        self.reservations.insert(self.next_reservation_id, reservation);
        self.reserve_seat(date, flight_id, seat_id);

        let result = format!("reserve: {} {}", self.next_reservation_id, price);
        self.next_reservation_id += 1;
        result
    }

    fn process_cancel(
        &mut self,
        current_datetime: &str,
        user_id: &str,
        reservation_id: u32,
    ) -> String {
        {
            let reservation = match self.reservations.get(&reservation_id) {
                Some(reservation) => reservation,
                None => return "cancel: reservation not found".to_string(),
            };

            if reservation.is_cancelled {
                return "cancel: reservation not found".to_string();
            }

            if reservation.user_id != user_id {
                return "cancel: unauthorized operation".to_string();
            }

            let flight = self.flights.get(&reservation.flight_id).unwrap();
            let current_dt = match NaiveDateTime::parse_from_str(current_datetime, "%Y/%m/%d-%H:%M:%S")
            {
                Ok(dt) => dt,
                Err(_) => return "cancel: invalid datetime".to_string(),
            };

            let flight_dt = match self.get_flight_datetime(&reservation.date, flight) {
                Some(dt) => dt,
                None => return "cancel: invalid flight datetime".to_string(),
            };

            if self.is_too_late(current_dt, flight_dt) {
                return "cancel: too late".to_string();
            }

        }

        let reservation_mut = self.reservations.get_mut(&reservation_id).unwrap();
        reservation_mut.is_cancelled = true;

        let date = reservation_mut.date.clone();
        let flight_id = reservation_mut.flight_id;
        let seat_id = reservation_mut.seat_id.clone();

        self.unreserve_seat(&date, flight_id, &seat_id);

        "cancel: success".to_string()
    }

    fn process_seat_search(
        &self,
        _current_datetime: &str,
        date: &str,
        flight_id: u32,
    ) -> String {
        if !self.flights.contains_key(&flight_id) {
            return "seat-search: flight not found".to_string();
        }

        let flight = self.flights.get(&flight_id).unwrap();
        let mut result = vec!["seat-search:".to_string()];
        let mut seats = vec![];

        for row in 1..=20 {
            let mut row_seats = HashMap::new();
            for seat_type in &SeatType::variants() {
                let seat_id = format!("{}{}", row, seat_type.as_char());
                let seat_display = if self.is_seat_reserved(date, flight_id, &seat_id) {
                    "X".to_string()
                } else {
                    let (seat_class, _) = flight.get_seat_class(&seat_id).unwrap();
                    seat_class.to_string()
                };
                row_seats.insert(seat_type.clone(), seat_display);
            }
            seats.push(row_seats);
        }

        for seat_type in &SeatType::variants() {
            let mut row_display = String::new();
            for row in &seats {
                row_display.push_str(&row[seat_type]);
            }
            result.push(row_display);
        }

        result.join("\n")
    }

    fn process_get_reservations(&self, _current_datetime: &str, user_id: &str) -> String {
        let mut valid_reservations = vec![];

        for reservation in self.reservations.values() {
            if reservation.user_id == user_id && !reservation.is_cancelled {
                let flight = self.flights.get(&reservation.flight_id).unwrap();
                let flight_dt = self.get_flight_datetime(&reservation.date, flight).unwrap();
                valid_reservations.push((flight_dt, reservation.reservation_id, reservation));
            }
        }

        valid_reservations.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        let mut result = vec![format!("get-reservations: {}", valid_reservations.len())];

        for (_, _, reservation) in valid_reservations {
            let flight = self.flights.get(&reservation.flight_id).unwrap();
            result.push(format!(
                "reservation id: {}, price: {}, seat: {} {} {}, route: {} ({}) -> {} ({})",
                reservation.reservation_id,
                reservation.price,
                reservation.date,
                reservation.flight_id,
                reservation.seat_id,
                flight.departure_airport,
                flight.departure_time,
                flight.arrival_airport,
                flight.arrival_time
            ));
        }

        result.join("\n")
    }

    fn process_flight_search(
        &self,
        _current_datetime: &str,
        date: &str,
        departure_airport: u32,
        arrival_airport: u32,
    ) -> String {
        let mut matching_flights = vec![];

        for flight in self.flights.values() {
            if flight.departure_airport == departure_airport
                && flight.arrival_airport == arrival_airport
            {
                matching_flights.push((flight.departure_time.clone(), flight.flight_id, flight));
            }
        }

        matching_flights.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        let mut result = vec![format!("flight-search: {}", matching_flights.len())];

        for (_, _, flight) in matching_flights {
            result.push(format!(
                "{} {} {}",
                flight.flight_id, flight.departure_time, flight.arrival_time
            ));

            for (i, seat_class) in flight.seat_classes.iter().enumerate() {
                let mut seats_count = 0;
                let start_row = if i == 0 {
                    1
                } else {
                    flight.seat_classes[i - 1].column + 1
                };
                for row in start_row..=seat_class.column {
                    for seat_type in &SeatType::variants() {
                        let seat_id = format!("{}{}", row, seat_type.as_char());
                        if !self.is_seat_reserved(date, flight.flight_id, &seat_id) {
                            seats_count += 1;
                        }
                    }
                }
                result.push(format!(
                    "class {}: {} seats available. price = {}",
                    i + 1,
                    seats_count,
                    seat_class.price
                ));
            }
        }

        result.join("\n")
    }
}

fn main() {
    let stdin = io::stdin();
    let mut iterator = stdin.lock().lines();

    let n: u32 = iterator.next().unwrap().unwrap().trim().parse().unwrap();
    let mut system = ReservationSystem::new();

    for _ in 0..n {
        let mut parts: Vec<String> = vec![];
        while parts.len() < 5 {
            let line = iterator.next().unwrap().unwrap();
            parts.extend(line.trim().split_whitespace().map(|s| s.to_string()));
        }
        let flight_id: u32 = parts[0].parse().unwrap();
        let departure_airport: u32 = parts[1].parse().unwrap();
        let arrival_airport: u32 = parts[2].parse().unwrap();
        let dep_time = parts[3].clone();
        let arr_time = parts[4].clone();

        let s_line = iterator.next().unwrap().unwrap();
        let s: u32 = s_line.trim().parse().unwrap();

        let mut seat_classes = vec![];

        for _ in 0..s {
            let line = iterator.next().unwrap().unwrap();
            let mut parts = line.trim().split_whitespace();
            let column: u32 = parts.next().unwrap().parse().unwrap();
            let price: u32 = parts.next().unwrap().parse().unwrap();
            seat_classes.push(SeatClass { column, price });
        }

        system.add_flight(
            flight_id,
            departure_airport,
            arrival_airport,
            dep_time,
            arr_time,
            seat_classes,
        );
    }

    let m_line = iterator.next().unwrap().unwrap();
    let m: u32 = m_line.trim().parse().unwrap();

    for _ in 0..m {
        let line = iterator.next().unwrap().unwrap();
        let query_line = line.trim();
        let query = query_line
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let command = query[0].clone();

        if command == "reserve:" {
            if query.len() != 6 {
                println!("reserve: invalid query");
                continue;
            }
            let datetime = &query[1];
            let user_id = &query[2];
            let date = &query[3];
            let flight_id: u32 = query[4].parse().unwrap();
            let seat_id = &query[5];
            println!(
                "{}",
                system.process_reserve(datetime, user_id, date, flight_id, seat_id)
            );
        } else if command == "cancel:" {
            if query.len() != 4 {
                println!("cancel: invalid query");
                continue;
            }
            let datetime = &query[1];
            let user_id = &query[2];
            let reservation_id: u32 = query[3].parse().unwrap();
            println!(
                "{}",
                system.process_cancel(datetime, user_id, reservation_id)
            );
        } else if command == "seat-search:" {
            if query.len() != 4 {
                println!("seat-search: invalid query");
                continue;
            }
            let datetime = &query[1];
            let date = &query[2];
            let flight_id: u32 = query[3].parse().unwrap();
            println!("{}", system.process_seat_search(datetime, date, flight_id));
        } else if command == "get-reservations:" {
            if query.len() != 3 {
                println!("get-reservations: invalid query");
                continue;
            }
            let datetime = &query[1];
            let user_id = &query[2];
            println!("{}", system.process_get_reservations(datetime, user_id));
        } else if command == "flight-search:" {
            if query.len() != 5 {
                println!("flight-search: invalid query");
                continue;
            }
            let datetime = &query[1];
            let date = &query[2];
            let departure_airport: u32 = query[3].parse().unwrap();
            let arrival_airport: u32 = query[4].parse().unwrap();
            println!(
                "{}",
                system.process_flight_search(datetime, date, departure_airport, arrival_airport)
            );
        }
    }
}
