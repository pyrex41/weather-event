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
    , scheduledDate : String
    , departureLocation : Location
    , status : String
    }


type alias Alert =
    { alertType : String
    , bookingId : String
    , message : String
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
    , loading : Bool
    , error : Maybe String
    , newBookingForm : BookingForm
    , newStudentForm : StudentForm
    , websocketStatus : WebSocketStatus
    }


type alias BookingForm =
    { studentId : String
    , scheduledDate : String
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
    | DismissAlert String
    | Tick Time.Posix


type BookingFormField
    = StudentIdField
    | ScheduledDateField
    | LocationNameField
    | LocationLatField
    | LocationLonField


type StudentFormField
    = NameField
    | EmailField
    | PhoneField
    | TrainingLevelField
