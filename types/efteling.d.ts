export interface ApiResponse {
    Version:         string;
    TimeStamp:       Date;
    Disclaimer:      string;
    OpeningHours:    OpeningHours;
    Event:           Event;
    AttractionInfo:  AttractionInfo[];
    MaintenanceInfo: any[];
}

export interface AttractionInfo {
    Id:                     string;
    Name:                   string;
    Type:                   Type;
    Empire:                 Empire;
    State:                  State;
    IsTheaterShow?:         boolean;
    ShowTimes?:             ShowTime[];
    PastShowTimes?:         ShowTime[];
    OpeningTimes?:          OpeningTime[];
    WaitingTime?:           number;
    PastOpeningTimes?:      OpeningTime[];
    CrowdednessIndication?: CrowdednessIndication;
}

export enum CrowdednessIndication {
    Busy = "busy",
    Calm = "calm",
    Gesloten = "gesloten",
    Normal = "normal",
}

export enum Empire {
    Anderrijk = "Anderrijk",
    Bosrijk = "Bosrijk",
    EftelingHotel = "Efteling Hotel",
    Fantasierijk = "Fantasierijk",
    LoonscheLand = "Loonsche Land",
    Marerijk = "Marerijk",
    Reizenrijk = "Reizenrijk",
    Ruigrijk = "Ruigrijk",
}

export interface OpeningTime {
    Date:     Date;
    HourFrom: Date;
    HourTo:   Date;
}

export interface ShowTime {
    ShowDateTime:  Date;
    StartDateTime: Date;
    EndDateTime:   Date;
    Edition?:      string;
}

export enum State {
    Buitenbedrijf = "buitenbedrijf",
    Gesloten = "gesloten",
    Open = "open",
}

export enum Type {
    Attracties = "Attracties",
    EtenEnDrinken = "Eten en Drinken",
    ParkEvenementLocaties = "Park Evenement Locaties",
    ShowsEnEntertainment = "Shows en Entertainment",
    Souvenirwinkels = "Souvenirwinkels",
    Toiletten = "Toiletten",
}

export interface Event {
    TextId:    string;
    DateStart: Date;
    DateEnd:   Date;
    Theme:     string;
    Title:     Title;
}

export interface Title {
    NL: string;
    EN: string;
    DE: string;
    FR: string;
}

export interface OpeningHours {
    Date:           Date;
    Crowdedness:    number;
    HourFrom:       Date;
    HourTo:         Date;
    BusyIndication: string;
    SpecialEvent:   boolean;
    OpeningTimes:   OpeningHoursOpeningTime[];
}

export interface OpeningHoursOpeningTime {
    Crowdedness:    number;
    HourFrom:       Date;
    HourTo:         Date;
    BusyIndication: string;
    SpecialEvent:   boolean;
}
