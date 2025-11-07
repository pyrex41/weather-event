module Main exposing (main)

import Api
import Browser
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Json.Decode as Decode
import Time
import Types exposing (..)
import WebSocket


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { page = Dashboard
      , bookings = []
      , students = []
      , alerts = []
      , loading = False
      , error = Nothing
      , newBookingForm = emptyBookingForm
      , newStudentForm = emptyStudentForm
      , websocketStatus = Connecting
      }
    , Cmd.batch
        [ Api.getBookings GotBookings
        , Api.getStudents GotStudents
        ]
    )


emptyBookingForm : BookingForm
emptyBookingForm =
    { studentId = ""
    , scheduledDate = ""
    , locationName = ""
    , locationLat = ""
    , locationLon = ""
    }


emptyStudentForm : StudentForm
emptyStudentForm =
    { name = ""
    , email = ""
    , phone = ""
    , trainingLevel = "STUDENT_PILOT"
    }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        ChangePage page ->
            ( { model | page = page }, Cmd.none )

        GotBookings result ->
            case result of
                Ok bookings ->
                    ( { model | bookings = bookings, loading = False }, Cmd.none )

                Err err ->
                    ( { model | error = Just err, loading = False }, Cmd.none )

        GotStudents result ->
            case result of
                Ok students ->
                    ( { model | students = students, loading = False }, Cmd.none )

                Err err ->
                    ( { model | error = Just err, loading = False }, Cmd.none )

        CreateBooking ->
            ( { model | loading = True }
            , Api.createBooking model.newBookingForm BookingCreated
            )

        BookingCreated result ->
            case result of
                Ok booking ->
                    ( { model
                        | bookings = booking :: model.bookings
                        , newBookingForm = emptyBookingForm
                        , loading = False
                      }
                    , Cmd.none
                    )

                Err err ->
                    ( { model | error = Just err, loading = False }, Cmd.none )

        CreateStudent ->
            ( { model | loading = True }
            , Api.createStudent model.newStudentForm StudentCreated
            )

        StudentCreated result ->
            case result of
                Ok student ->
                    ( { model
                        | students = student :: model.students
                        , newStudentForm = emptyStudentForm
                        , loading = False
                      }
                    , Cmd.none
                    )

                Err err ->
                    ( { model | error = Just err, loading = False }, Cmd.none )

        UpdateBookingForm field value ->
            let
                form =
                    model.newBookingForm

                newForm =
                    case field of
                        StudentIdField ->
                            { form | studentId = value }

                        ScheduledDateField ->
                            { form | scheduledDate = value }

                        LocationNameField ->
                            { form | locationName = value }

                        LocationLatField ->
                            { form | locationLat = value }

                        LocationLonField ->
                            { form | locationLon = value }
            in
            ( { model | newBookingForm = newForm }, Cmd.none )

        UpdateStudentForm field value ->
            let
                form =
                    model.newStudentForm

                newForm =
                    case field of
                        NameField ->
                            { form | name = value }

                        EmailField ->
                            { form | email = value }

                        PhoneField ->
                            { form | phone = value }

                        TrainingLevelField ->
                            { form | trainingLevel = value }
            in
            ( { model | newStudentForm = newForm }, Cmd.none )

        WebSocketMessageReceived message ->
            case Decode.decodeString Api.alertDecoder message of
                Ok alert ->
                    ( { model | alerts = alert :: model.alerts }, Cmd.none )

                Err _ ->
                    ( model, Cmd.none )

        WebSocketConnected ->
            ( { model | websocketStatus = Connected }, Cmd.none )

        WebSocketDisconnected ->
            ( { model | websocketStatus = Disconnected }, Cmd.none )

        DismissAlert bookingId ->
            ( { model | alerts = List.filter (\a -> a.bookingId /= bookingId) model.alerts }
            , Cmd.none
            )

        Tick _ ->
            ( model, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        [ WebSocket.websocketIn WebSocketMessageReceived
        , WebSocket.websocketConnected (\_ -> WebSocketConnected)
        , WebSocket.websocketDisconnected (\_ -> WebSocketDisconnected)
        , Time.every 10000 Tick
        ]


view : Model -> Html Msg
view model =
    div [ class "app" ]
        [ viewHeader model
        , viewNavigation model
        , viewAlerts model
        , viewContent model
        ]


viewHeader : Model -> Html Msg
viewHeader model =
    header [ class "header" ]
        [ h1 [] [ text "✈️ Flight Schedule Pro" ]
        , div [ class "status" ]
            [ viewWebSocketStatus model.websocketStatus ]
        ]


viewWebSocketStatus : WebSocketStatus -> Html Msg
viewWebSocketStatus status =
    case status of
        Connecting ->
            span [ class "status-badge connecting" ] [ text "Connecting..." ]

        Connected ->
            span [ class "status-badge connected" ] [ text "● Live" ]

        Disconnected ->
            span [ class "status-badge disconnected" ] [ text "○ Disconnected" ]


viewNavigation : Model -> Html Msg
viewNavigation model =
    nav [ class "navigation" ]
        [ button
            [ class (navClass model Dashboard)
            , onClick (ChangePage Dashboard)
            ]
            [ text "Dashboard" ]
        , button
            [ class (navClass model Bookings)
            , onClick (ChangePage Bookings)
            ]
            [ text "Bookings" ]
        , button
            [ class (navClass model Students)
            , onClick (ChangePage Students)
            ]
            [ text "Students" ]
        , button
            [ class (navClass model Alerts)
            , onClick (ChangePage Alerts)
            ]
            [ text "Alerts" ]
        ]


navClass : Model -> Page -> String
navClass model page =
    if model.page == page then
        "nav-button active"

    else
        "nav-button"


viewAlerts : Model -> Html Msg
viewAlerts model =
    if List.isEmpty model.alerts then
        text ""

    else
        div [ class "alerts-banner" ]
            (List.map viewAlert model.alerts)


viewAlert : Alert -> Html Msg
viewAlert alert =
    div [ class "alert alert-danger" ]
        [ span [] [ text ("⚠️ " ++ alert.message) ]
        , button
            [ class "alert-dismiss"
            , onClick (DismissAlert alert.bookingId)
            ]
            [ text "×" ]
        ]


viewContent : Model -> Html Msg
viewContent model =
    div [ class "content" ]
        [ case model.error of
            Just err ->
                div [ class "error" ] [ text ("Error: " ++ err) ]

            Nothing ->
                text ""
        , case model.page of
            Dashboard ->
                viewDashboard model

            Bookings ->
                viewBookings model

            Students ->
                viewStudents model

            Alerts ->
                viewAlertsPage model
        ]


viewDashboard : Model -> Html Msg
viewDashboard model =
    div [ class "dashboard" ]
        [ h2 [] [ text "Dashboard" ]
        , div [ class "dashboard-stats" ]
            [ div [ class "stat-card" ]
                [ h3 [] [ text (String.fromInt (List.length model.bookings)) ]
                , p [] [ text "Total Bookings" ]
                ]
            , div [ class "stat-card" ]
                [ h3 [] [ text (String.fromInt (List.length model.students)) ]
                , p [] [ text "Students" ]
                ]
            , div [ class "stat-card" ]
                [ h3 [] [ text (String.fromInt (List.length model.alerts)) ]
                , p [] [ text "Active Alerts" ]
                ]
            ]
        , h3 [] [ text "Recent Bookings" ]
        , viewBookingsList (List.take 5 model.bookings)
        ]


viewBookings : Model -> Html Msg
viewBookings model =
    div [ class "bookings-page" ]
        [ h2 [] [ text "Bookings" ]
        , div [ class "form-card" ]
            [ h3 [] [ text "Create New Booking" ]
            , viewBookingForm model
            ]
        , h3 [] [ text "All Bookings" ]
        , viewBookingsList model.bookings
        ]


viewBookingForm : Model -> Html Msg
viewBookingForm model =
    div [ class "form" ]
        [ div [ class "form-group" ]
            [ label [] [ text "Student" ]
            , select
                [ onInput (UpdateBookingForm StudentIdField)
                , value model.newBookingForm.studentId
                ]
                (option [ value "" ] [ text "Select a student" ]
                    :: List.map
                        (\s -> option [ value s.id ] [ text s.name ])
                        model.students
                )
            ]
        , div [ class "form-group" ]
            [ label [] [ text "Scheduled Date (YYYY-MM-DDTHH:MM:SSZ)" ]
            , input
                [ type_ "text"
                , placeholder "2024-01-15T14:00:00Z"
                , value model.newBookingForm.scheduledDate
                , onInput (UpdateBookingForm ScheduledDateField)
                ]
                []
            ]
        , div [ class "form-group" ]
            [ label [] [ text "Location Name" ]
            , input
                [ type_ "text"
                , placeholder "KTOA"
                , value model.newBookingForm.locationName
                , onInput (UpdateBookingForm LocationNameField)
                ]
                []
            ]
        , div [ class "form-row" ]
            [ div [ class "form-group" ]
                [ label [] [ text "Latitude" ]
                , input
                    [ type_ "text"
                    , placeholder "33.8113"
                    , value model.newBookingForm.locationLat
                    , onInput (UpdateBookingForm LocationLatField)
                    ]
                    []
                ]
            , div [ class "form-group" ]
                [ label [] [ text "Longitude" ]
                , input
                    [ type_ "text"
                    , placeholder "-118.1515"
                    , value model.newBookingForm.locationLon
                    , onInput (UpdateBookingForm LocationLonField)
                    ]
                    []
                ]
            ]
        , button
            [ class "button button-primary"
            , onClick CreateBooking
            , disabled model.loading
            ]
            [ text
                (if model.loading then
                    "Creating..."

                 else
                    "Create Booking"
                )
            ]
        ]


viewBookingsList : List Booking -> Html Msg
viewBookingsList bookings =
    if List.isEmpty bookings then
        p [] [ text "No bookings found." ]

    else
        div [ class "bookings-list" ]
            (List.map viewBookingCard bookings)


viewBookingCard : Booking -> Html Msg
viewBookingCard booking =
    div [ class ("booking-card status-" ++ String.toLower booking.status) ]
        [ div [ class "booking-header" ]
            [ h4 [] [ text booking.departureLocation.name ]
            , span [ class "status-badge" ] [ text booking.status ]
            ]
        , p [] [ text ("Date: " ++ formatDateWithTimezone booking.scheduledDate) ]
        , p [] [ text ("Student ID: " ++ booking.studentId) ]
        ]


formatDateWithTimezone : String -> String
formatDateWithTimezone dateStr =
    dateStr ++ " UTC"


viewStudents : Model -> Html Msg
viewStudents model =
    div [ class "students-page" ]
        [ h2 [] [ text "Students" ]
        , div [ class "form-card" ]
            [ h3 [] [ text "Add New Student" ]
            , viewStudentForm model
            ]
        , h3 [] [ text "All Students" ]
        , viewStudentsList model.students
        ]


viewStudentForm : Model -> Html Msg
viewStudentForm model =
    div [ class "form" ]
        [ div [ class "form-group" ]
            [ label [] [ text "Name" ]
            , input
                [ type_ "text"
                , placeholder "John Doe"
                , value model.newStudentForm.name
                , onInput (UpdateStudentForm NameField)
                ]
                []
            ]
        , div [ class "form-group" ]
            [ label [] [ text "Email" ]
            , input
                [ type_ "email"
                , placeholder "john@example.com"
                , value model.newStudentForm.email
                , onInput (UpdateStudentForm EmailField)
                ]
                []
            ]
        , div [ class "form-group" ]
            [ label [] [ text "Phone" ]
            , input
                [ type_ "tel"
                , placeholder "+1234567890"
                , value model.newStudentForm.phone
                , onInput (UpdateStudentForm PhoneField)
                ]
                []
            ]
        , div [ class "form-group" ]
            [ label [] [ text "Training Level" ]
            , select
                [ onInput (UpdateStudentForm TrainingLevelField)
                , value model.newStudentForm.trainingLevel
                ]
                [ option [ value "STUDENT_PILOT" ] [ text "Student Pilot" ]
                , option [ value "PRIVATE_PILOT" ] [ text "Private Pilot" ]
                , option [ value "INSTRUMENT_RATED" ] [ text "Instrument Rated" ]
                ]
            ]
        , button
            [ class "button button-primary"
            , onClick CreateStudent
            , disabled model.loading
            ]
            [ text
                (if model.loading then
                    "Creating..."

                 else
                    "Add Student"
                )
            ]
        ]


viewStudentsList : List Student -> Html Msg
viewStudentsList students =
    if List.isEmpty students then
        p [] [ text "No students found." ]

    else
        div [ class "students-list" ]
            (List.map viewStudentCard students)


viewStudentCard : Student -> Html Msg
viewStudentCard student =
    div [ class "student-card" ]
        [ h4 [] [ text student.name ]
        , p [] [ text ("Email: " ++ student.email) ]
        , p [] [ text ("Phone: " ++ student.phone) ]
        , span [ class "badge" ] [ text student.trainingLevel ]
        ]


viewAlertsPage : Model -> Html Msg
viewAlertsPage model =
    div [ class "alerts-page" ]
        [ h2 [] [ text "Weather Alerts" ]
        , if List.isEmpty model.alerts then
            p [] [ text "No active alerts." ]

          else
            div [ class "alerts-list" ]
                (List.map viewAlertCard model.alerts)
        ]


viewAlertCard : Alert -> Html Msg
viewAlertCard alert =
    div [ class "alert-card" ]
        [ h4 [] [ text ("⚠️ " ++ alert.alertType) ]
        , p [] [ text alert.message ]
        , case alert.studentName of
            Just name ->
                p [] [ text ("Student: " ++ name) ]

            Nothing ->
                text ""
        , case alert.originalDate of
            Just date ->
                p [] [ text ("Original Date: " ++ formatDateWithTimezone date) ]

            Nothing ->
                text ""
        , button
            [ class "button button-secondary"
            , onClick (DismissAlert alert.bookingId)
            ]
            [ text "Dismiss" ]
        ]
