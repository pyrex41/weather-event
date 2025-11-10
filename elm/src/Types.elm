module Types exposing (..)

import Time


type alias Location =
    { lat : Float
    , lon : Float
    , name : String
    }


type alias Student =
    { id : String
    , name : String
    , email : String
    , phone : String
    , trainingLevel : String
    }


type alias Booking =
    { id : String
    , studentId : String
    , aircraftType : String
    , scheduledDate : String
    , departureLocation : Location
    , status : String
    }


type Severity
    = Severe
    | High
    | Moderate
    | Low
    | Clear


type alias Alert =
    { id : String
    , alertType : String
    , bookingId : String
    , message : String
    , severity : Severity
    , location : String
    , timestamp : String
    , studentName : Maybe String
    , originalDate : Maybe String
    }


type alias RescheduleOption =
    { dateTime : String
    , reason : String
    , weatherScore : Float
    , instructorAvailable : Bool
    }


type Page
    = Dashboard
    | Bookings
    | Students
    | Alerts


type alias Model =
    { page : Page
    , bookings : List Booking
    , students : List Student
    , alerts : List Alert
    , bookingsLoading : Bool
    , studentsLoading : Bool
    , alertsLoading : Bool
    , formSubmitting : Bool
    , error : Maybe String
    , successMessage : Maybe String
    , successMessageTime : Maybe Time.Posix
    , bookingFormErrors : List FormError
    , studentFormErrors : List FormError
    , newBookingForm : BookingForm
    , newStudentForm : StudentForm
    , websocketStatus : WebSocketStatus
    , rescheduleModal : Maybe RescheduleModal
    }


type alias RescheduleModal =
    { booking : Booking
    , options : List RescheduleOption
    , loading : Bool
    , selectedOption : Maybe RescheduleOption
    , showConfirmation : Bool
    }


type alias BookingForm =
    { studentId : String
    , aircraftType : String
    , scheduledDate : String
    , endTime : String
    , locationName : String
    , locationLat : String
    , locationLon : String
    }


type alias StudentForm =
    { name : String
    , email : String
    , phone : String
    , trainingLevel : String
    }


type WebSocketStatus
    = Connecting
    | Connected
    | Disconnected


type alias FormError =
    { field : String
    , message : String
    }


type Msg
    = ChangePage Page
    | GotBookings (Result String (List Booking))
    | GotStudents (Result String (List Student))
    | CreateBooking
    | BookingCreated (Result String Booking)
    | CreateStudent
    | StudentCreated (Result String Student)
    | UpdateBookingForm BookingFormField String
    | UpdateStudentForm StudentFormField String
    | WebSocketMessageReceived String
    | WebSocketConnected
    | WebSocketDisconnected
    | DismissAlert String  -- Alert ID
    | ClearSuccessMessage
    | SetSuccessMessage String Time.Posix
    | Tick Time.Posix
    | OpenRescheduleModal Booking
    | CloseRescheduleModal
    | GotRescheduleOptions (Result String (List RescheduleOption))
    | SelectRescheduleOption RescheduleOption
    | ShowRescheduleConfirmation
    | CancelRescheduleConfirmation
    | ConfirmReschedule
    | RescheduleCompleted (Result String Booking)


type BookingFormField
    = StudentIdField
    | AircraftTypeField
    | ScheduledDateField
    | EndTimeField
    | LocationNameField
    | LocationLatField
    | LocationLonField


type StudentFormField
    = NameField
    | EmailField
    | PhoneField
    | TrainingLevelField
