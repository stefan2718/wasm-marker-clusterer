describe('Library API', function() {
  it('runs the library', function() {
    cy.visit("/");
    cy.get('.loaded').contains("Libary loaded");
    cy.get('.clusters-length').contains(1);
    cy.get('.cluster-size').contains(1);
    cy.get('.clusters-length-2').contains(0);
  })
})