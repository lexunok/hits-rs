    // public Flux<ProjectDTO> getAllProjectsForTeam(String teamId){
    //     String QUERY = """
    //             SELECT
    //                 p.id AS p_id, p.idea_id AS p_idea_id, p.start_date AS p_start_date, p.finish_date AS p_finish_date, p.status AS p_status,
    //                 i.name AS i_name, i.description AS i_description, i.customer AS i_customer,
    //                 u.id AS u_id, u.email AS u_email, u.first_name AS u_first_name, u.last_name AS u_last_name
    //             FROM project p
    //                 LEFT JOIN idea i ON i.id = p.idea_id
    //                 LEFT JOIN users u ON u.id = i.initiator_id
    //             WHERE p.team_id = :teamId
    //             """;
    //     return template.getDatabaseClient()
    //             .sql(QUERY)
    //             .bind("teamId", teamId)
    //             .map((row, rowMetadata) -> new ProjectDTO(
    //                     row.get("p_id", String.class),
    //                     row.get("p_idea_id", String.class),
    //                     row.get("i_name", String.class),
    //                     row.get("i_description", String.class),
    //                     row.get("i_customer", String.class),
    //                     UserDTO.builder()
    //                             .id(row.get("u_id", String.class))
    //                             .email(row.get("u_email", String.class))
    //                             .firstName(row.get("u_first_name", String.class))
    //                             .lastName(row.get("u_last_name", String.class))
    //                             .build(),
    //                     null,
    //                     null,
    //                     null,
    //                     row.get("p_start_date", LocalDate.class),
    //                     row.get("p_finish_date", LocalDate.class),
    //                     ProjectStatus.valueOf(row.get("p_status", String.class)))
    //             )
    //             .all();
    // }