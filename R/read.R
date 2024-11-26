#' Read and parse 'tenhou.net/6' format log
#'
#' Read and parse 'tenhou.net/6' format JSON files
#' while transforming them into 'mjai' format.
#'
#' `read_mjlog()` internally reads remote JSON files corresponding to `logid`,
#' and converts them into the same format as `read_tenhou6()`.
#'
#' Note that `read_mjlog()` is rate-limited to 2 requests per second
#' to access the server.
#'
#' @rdname read-tenhou6
#' @name read-tenhou6
#' @param file A character vector.
#' This argument is simply passed to `scan()`,
#' so each element can be either a path to a local file or a URL.
#' @param logid A character vector that represents identifiers of log files.
#' @param .progress Whether to show progress bar for `purrr::map_chr()`.
#' @returns A named list that contains following elements:
#' * `game_info`: A tibble that contains information about the games.
#' * `round_info`: A tibble that contains information about rounds.
#' * `paifu`: A tibble that represents paifu.
NULL

#' @rdname read-tenhou6
#' @export
read_tenhou6 <- function(file, .progress = FALSE) {
  purrr::map_chr(file, function(elem) {
    scan(elem, what = character(), sep = "\n", quiet = TRUE)
  }, .progress = .progress) |>
    parse_tenhou6() |>
    parse_mjai()
}

#' @rdname read-tenhou6
#' @export
read_mjlog <- function(logid, .progress = FALSE) {
  logid <- paste0("https://tenhou.net/5/mjlog2json.cgi?", logid)
  purrr::map_chr(logid, function(id) {
    scan_ltd(url(id, headers = c("Referer" = "https://tenhou.net/")), what = character(), sep = "\n", quiet = TRUE)
  }, .progress = .progress) |>
    parse_tenhou6() |>
    parse_mjai()
}
