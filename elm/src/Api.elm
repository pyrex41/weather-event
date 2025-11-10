module Api exposing (..)

import Http
import Json.Decode as Decode exposing (Decoder)
import Json.Encode as Encode
import Types exposing (..)


apiUrl : String -> String
apiUrl path =
    "/api" ++ path


locationDecoder : Decoder Location
locationDecoder =
    Decode.map3 Location
        (Decode.field "lat" Decode.float)
        (Decode.field "lon" Decode.float)
        (Decode.field "name" Decode.string)


bookingDecoder : Decoder Booking
bookingDecoder =
    Decode.map6 Booking
        (Decode.field "id" Decode.string)
        (Decode.field "student_id" Decode.string)
        (Decode.field "aircraft_type" Decode.string)
        (Decode.field "scheduled_date" Decode.string)
        (Decode.field "departure_location" locationDecoder)
        (Decode.field "status" Decode.string)


studentDecoder : Decoder Student
studentDecoder =
    Decode.map5 Student
        (Decode.field "id" Decode.string)
        (Decode.field "name" Decode.string)
        (Decode.field "email" Decode.string)
        (Decode.field "phone" Decode.string)
        (Decode.field "training_level" Decode.string)


severityDecoder : Decoder Severity
severityDecoder =
    Decode.string
        |> Decode.andThen
            (\str ->
                case String.toLower str of
                    "severe" ->
                        Decode.succeed Severe

                    "high" ->
                        Decode.succeed High

                    "moderate" ->
                        Decode.succeed Moderate

                    "low" ->
                        Decode.succeed Low

                    "clear" ->
                        Decode.succeed Clear

                    _ ->
                        Decode.succeed Moderate
            )


alertDecoder : Decoder Alert
alertDecoder =
    Decode.succeed Alert
        |> andMap (Decode.oneOf [ Decode.field "id" Decode.string, Decode.field "booking_id" Decode.string ])
        |> andMap (Decode.field "type" Decode.string)
        |> andMap (Decode.field "booking_id" Decode.string)
        |> andMap (Decode.field "message" Decode.string)
        |> andMap (Decode.oneOf [ Decode.field "severity" severityDecoder, Decode.succeed Moderate ])
        |> andMap (Decode.oneOf [ Decode.field "location" Decode.string, Decode.succeed "" ])
        |> andMap (Decode.oneOf [ Decode.field "timestamp" Decode.string, Decode.succeed "" ])
        |> andMap (Decode.maybe (Decode.field "student_name" Decode.string))
        |> andMap (Decode.maybe (Decode.field "original_date" Decode.string))


andMap : Decoder a -> Decoder (a -> b) -> Decoder b
andMap =
    Decode.map2 (|>)


getBookings : (Result String (List Booking) -> msg) -> Cmd msg
getBookings toMsg =
    Http.get
        { url = apiUrl "/bookings"
        , expect = expectJson toMsg (Decode.list bookingDecoder)
        }


getStudents : (Result String (List Student) -> msg) -> Cmd msg
getStudents toMsg =
    Http.get
        { url = apiUrl "/students"
        , expect = expectJson toMsg (Decode.list studentDecoder)
        }


createBooking : BookingForm -> (Result String Booking -> msg) -> Cmd msg
createBooking form toMsg =
    let
        body =
            Encode.object
                [ ( "student_id", Encode.string form.studentId )
                , ( "aircraft_type", Encode.string form.aircraftType )
                , ( "scheduled_date", Encode.string form.scheduledDate )
                , ( "departure_location"
                  , Encode.object
                        [ ( "lat", Encode.float (String.toFloat form.locationLat |> Maybe.withDefault 0) )
                        , ( "lon", Encode.float (String.toFloat form.locationLon |> Maybe.withDefault 0) )
                        , ( "name", Encode.string form.locationName )
                        ]
                  )
                ]
    in
    Http.post
        { url = apiUrl "/bookings"
        , body = Http.jsonBody body
        , expect = expectJson toMsg bookingDecoder
        }


createStudent : StudentForm -> (Result String Student -> msg) -> Cmd msg
createStudent form toMsg =
    let
        body =
            Encode.object
                [ ( "name", Encode.string form.name )
                , ( "email", Encode.string form.email )
                , ( "phone", Encode.string form.phone )
                , ( "training_level", Encode.string form.trainingLevel )
                ]
    in
    Http.post
        { url = apiUrl "/students"
        , body = Http.jsonBody body
        , expect = expectJson toMsg studentDecoder
        }


rescheduleOptionDecoder : Decoder RescheduleOption
rescheduleOptionDecoder =
    Decode.map4 RescheduleOption
        (Decode.field "date_time" Decode.string)
        (Decode.field "reason" Decode.string)
        (Decode.field "weather_score" Decode.float)
        (Decode.field "instructor_available" Decode.bool)


getRescheduleSuggestions : String -> (Result String (List RescheduleOption) -> msg) -> Cmd msg
getRescheduleSuggestions bookingId toMsg =
    Http.get
        { url = apiUrl ("/bookings/" ++ bookingId ++ "/reschedule-suggestions")
        , expect = expectJson toMsg (Decode.field "options" (Decode.list rescheduleOptionDecoder))
        }


rescheduleBooking : String -> String -> (Result String Booking -> msg) -> Cmd msg
rescheduleBooking bookingId newDateTime toMsg =
    let
        body =
            Encode.object
                [ ( "new_scheduled_date", Encode.string newDateTime )
                ]
    in
    Http.request
        { method = "PATCH"
        , headers = []
        , url = apiUrl ("/bookings/" ++ bookingId ++ "/reschedule")
        , body = Http.jsonBody body
        , expect = expectJson toMsg bookingDecoder
        , timeout = Nothing
        , tracker = Nothing
        }


-- API Error response decoder
apiErrorDecoder : Decoder String
apiErrorDecoder =
    Decode.at [ "error", "message" ] Decode.string


expectJson : (Result String a -> msg) -> Decoder a -> Http.Expect msg
expectJson toMsg decoder =
    Http.expectStringResponse toMsg <|
        \response ->
            case response of
                Http.BadUrl_ url ->
                    Err ("Bad URL: " ++ url)

                Http.Timeout_ ->
                    Err "Request timed out. Please try again."

                Http.NetworkError_ ->
                    Err "Network error. Please check your connection."

                Http.BadStatus_ metadata body ->
                    -- Try to parse structured API error response first
                    case Decode.decodeString apiErrorDecoder body of
                        Ok errorMessage ->
                            Err errorMessage

                        Err _ ->
                            -- Fallback to generic HTTP error
                            Err ("HTTP " ++ String.fromInt metadata.statusCode ++ ": Request failed")

                Http.GoodStatus_ _ body ->
                    case Decode.decodeString decoder body of
                        Ok value ->
                            Ok value

                        Err err ->
                            Err ("Failed to parse response: " ++ Decode.errorToString err)
