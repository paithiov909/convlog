#' @importFrom ratelimitr limit_rate rate
scan_ltd <- ratelimitr::limit_rate(scan, ratelimitr::rate(n = 2, period = 1))

wh0 <- function(x, ...) {
  which(x, ...) - 1
}

#' Parse mjai log
#'
#' @param list_chr A list of character vectors out of `parse_tenhou6`.
#' @returns A list.
#' @importFrom RcppSimdJson fparse
#' @noRd
parse_mjai <- function(list_chr) {
  purrr::imap(list_chr, function(json, game_id) {
    collapsed <- paste0("[", paste0(json, collapse = ","), "]")

    # At first, get types for all events
    types <-
      fparse(collapsed, query = paste0("/", seq_along(json) - 1, "/type"))

    # Then, parse separately while grouping by type
    meta_event <- c("start_game", "start_kyoku", "end_kyoku", "end_game")
    start_game <-
      fparse(
        collapsed,
        query = paste0("/", wh0(types == "start_game"))
      )
    start_kyoku <-
      fparse(
        collapsed,
        query = paste0("/", wh0(types == "start_kyoku")),
        always_list = FALSE
      )

    # when there is only one kyoku, parsed result is simplified by fparse
    # so we need to make it one more level nested.
    if ("type" %in% names(start_kyoku)) {
      start_kyoku <- list(start_kyoku)
    }
    start_kyoku <- start_kyoku |>
      purrr::list_transpose() |>
      tibble::as_tibble()

    other_events <-
      c(
        # for template, create an union of all possible events
        list(c(
          type = NA_character_,
          actor = NA_integer_,
          target = NA_integer_,
          pai = NA_character_,
          tsumogiri = NA,
          consumed = list(NA_character_),
          dora_marker = NA_character_, # dora
          deltas = list(NA_real_), # hora, ryukyoku. optional
          ura_markers = list(NA_character_) # hora. optional
        )),
        fparse(collapsed, query = paste0("/", wh0(!types %in% meta_event)))
      ) |>
      purrr::list_transpose(
        default = list(
          type = NA_character_,
          actor = NA_integer_,
          target = NA_integer_,
          pai = NA_character_,
          tsumogiri = NA,
          consumed = NULL,
          dora_marker = NA_character_,
          deltas = NULL,
          ura_markers = NULL
        )
      ) |>
      tibble::as_tibble()

    round_id <- with(
      rle(types %in% meta_event),
      rep(seq_along(values), lengths)
    )[!types %in% meta_event]

    list(
      game_info = tibble::tibble(
        game_id = game_id,
        names = start_game["names"],
        qijia = start_game[["kyoku_first"]],
        aka = start_game[["aka_flag"]]
      ),
      round_info = tibble::tibble(
        game_id = game_id,
        round_id = seq_along(unique(round_id)),
        start_kyoku[-1] # remove type column
      ),
      paifu = tibble::tibble(
        game_id = game_id,
        round_id = with(
          rle(round_id),
          rep(seq_along(values), lengths)
        ),
        tibble::rowid_to_column(
          other_events[2:nrow(other_events), ],
          "event_id"
        )
      )
    )
  })
}
