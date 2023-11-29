#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <cmath>
using namespace std;

const double PI = 4.0 * atan( 1.0 );
class STL;

//======================================================================

struct vec
{
   double x, y, z;
};

//----------------------------

vec operator + ( const vec a, const vec b ){ return { a.x + b.x, a.y + b.y, a.z + b.z }; }
vec operator - ( const vec a, const vec b ){ return { a.x - b.x, a.y - b.y, a.z - b.z }; }
vec operator / ( const vec a, double d    ){ return { a.x / d, a.y / d, a.z / d }; }
vec operator * ( double d   , const vec a ){ return { d * a.x, d * a.y, d * a.z }; }
vec operator * ( const vec a, const vec b ){ return { a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x }; }
double normsq  ( const vec a              ){ return a.x * a.x + a.y * a.y + a.z * a.z; }
double len     ( const vec a              ){ return sqrt( normsq( a ) ); }
ostream & operator << ( ostream &out, const vec a ){ return out << a.x << " " << a.y << " " << a.z << " "; }

//======================================================================

struct Triangle
{
   vec v1, v2, v3;
};

//======================================================================

class Shape
{
public:
   virtual void addToPlot( STL &stl ) = 0;
};

//======================================================================

class STL
{
   vector<Triangle> triangles;
public:
   void add( Shape *S );
   void addTriangle( vec v1, vec v2, vec v3 );
   void addRectangle( vec v1, vec v2, vec v3, vec v4 );
   void draw( const string &filename );
};

//----------------------------

void STL::add( Shape *S )
{
   S->addToPlot( *this );
}

//----------------------------

void STL::addTriangle( vec v1, vec v2, vec v3 )
{
   triangles.push_back( { v1, v2, v3 } );
}

//----------------------------

void STL::addRectangle( vec v1, vec v2, vec v3, vec v4 )
{
   addTriangle( v1, v2, v3 );
   addTriangle( v1, v3, v4 );
}

//----------------------------

void STL::draw( const string &filename )
{
   ofstream out( filename );
   out << "solid\n";

   for ( Triangle tri : triangles )
   {
      vec n = ( tri.v2 - tri.v1 ) * ( tri.v3 - tri.v1 );
      n = n / len( n );                // unit normal vector;

      out << "   facet normal " << n << '\n';
      out << "      outer loop\n";
      out << "         vertex " << tri.v1 << '\n';
      out << "         vertex " << tri.v2 << '\n';
      out << "         vertex " << tri.v3 << '\n';
      out << "      endloop\n";
      out << "   endfacet\n";
   }

   out << "endsolid\n";
}

//======================================================================

class Cube : public Shape
{
   vec centre;                         // centre
   double L;                           // side length
   vec side1, side2, side3;            // TO DO: needs general orientation

public:
   Cube( vec centre, double L ) : centre( centre ), L( L )       // Specify centre and side length
   {
      side1 = vec{ L, 0, 0 };
      side2 = vec{ 0, L, 0 };
      side3 = vec{ 0, 0, L };
   }
   Cube( vec centre, vec side1, vec side2 ) : centre( centre ), side1( side1 ), side2( side2 )
   {                                                             // Specify centre and two side vectors
      L = len( side1 );
      side3 = side1 * side2 / L;
   }
   void addToPlot( STL &stl );
};

//----------------------------

void Cube::addToPlot( STL &stl )
{
   vec v1 = centre - 0.5 * ( side1 + side2 + side3 );
   vec v2 = v1 + side1;
   vec v3 = v2 + side2;
   vec v4 = v3 - side1;
   vec v5 = v1 + side3, v6 = v2 + side3, v7 = v3 + side3, v8 = v4 + side3;
   stl.addRectangle( v1, v2, v6, v5 );
   stl.addRectangle( v2, v3, v7, v6 );
   stl.addRectangle( v3, v4, v8, v7 );
   stl.addRectangle( v4, v1, v5, v8 );
   stl.addRectangle( v1, v4, v3, v2 );
   stl.addRectangle( v5, v6, v7, v8 );
}

//======================================================================

class Cylinder : public Shape
{
   vec centre;                                             // centre (NOT base centre)
   double r;                                               // radius
   double h;                                               // height
   int nface;                                              // number of rectangular faces
   vec side1, side2, side3;                                // vectors along radius, along radius perpendicular, along axis

public:
   Cylinder( vec centre, double r, double h, int n = 30 ) : centre( centre ), r( r ), h( h ), nface( n )
   {                                                       // Specify centre, radius, height
      side1 = vec{ r, 0, 0 };
      side2 = vec{ 0, r, 0 };
      side3 = vec{ 0, 0, h };
   }
   Cylinder( vec centre, vec side1, vec side3, int n = 30 ) : centre( centre ), side1( side1 ), side3( side3 ), nface(n)
   {                                                       // Specify vectors for centre, radius, height
      r = len( side1 );
      h = len( side3 );
      side2 = side3 * side1 / h;
   }
   void addToPlot( STL &stl );
};

//----------------------------

void Cylinder::addToPlot( STL &stl )
{
   vec bottom = centre - 0.5 * side3;                      // centre of base
   vec top    = bottom + side3;                            // centre of top

   double dtheta = 2.0 * PI / nface;
   vec v1, v2, v3, v4;
   v2 = bottom + side1;
   v3 = v2 + side3;
   for ( int n = 1; n <= nface; n++ )
   {
      double theta = n * dtheta;
      v1 = v2;
      v4 = v3;
      v2 = bottom + cos( theta ) * side1 + sin( theta ) * side2;
      v3 = v2 + side3;
      stl.addRectangle( v1, v2, v3, v4 );                  // add sides as a series of rectangles
      stl.addTriangle( v2, v1, bottom );                   // add triangles for bottom
      stl.addTriangle( v4, v3, top    );                   // add triangles for top
   }
}

//======================================================================

int main()
{
   Cube cube1( vec{  0, 0, 0 }, 25 );
   Cube cube2( vec{ 50, 0, 0 }, vec{ 20, 20, 0 }, vec{ 20, -20, 0 } );
   Cylinder cylinder1( vec{  0, 50, 0 }, 20, 30 );
   Cylinder cylinder2( vec{ 50, 50, 0 }, vec{ 20, 20, 0 }, vec{ 20, -20, 0 } );

   STL stl;
   stl.add( &cube1 );
   // stl.add( &cube2 );
   // stl.add( &cylinder1 );
   // stl.add( &cylinder2 );
   stl.draw( "stl.stl" );
}
