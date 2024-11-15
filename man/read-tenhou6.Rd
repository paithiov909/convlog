% Generated by roxygen2: do not edit by hand
% Please edit documentation in R/read.R
\name{read-tenhou6}
\alias{read-tenhou6}
\alias{read_tenhou6}
\alias{read_mjlog}
\title{Read and parse 'tenhou.net/6' format log}
\usage{
read_tenhou6(file)

read_mjlog(logid)
}
\arguments{
\item{file}{A character vector.
This argument is simply passed to \code{scan()},
so each element can be either a path to a local file or a URL.}

\item{logid}{A character vector that represents identifiers of log files.}
}
\value{
A list that contains following named elements:
\itemize{
\item \code{game_info}: A tibble that contains information about the game.
\item \code{round_info}: A tibble that contains information about rounds.
\item \code{paifu}: A tibble that represents paifu.
}
}
\description{
Read and parse 'tenhou.net/6' format JSON files
while transforming them into 'mjai' format.
}
\details{
\code{read_mjlog()} internally reads remote JSON files corresponding to \code{logid},
and converts them into the same format as \code{read_tenhou6()}.

Note that \code{read_mjlog()} is rate-limited to 2 requests per second
to access the server.
}
