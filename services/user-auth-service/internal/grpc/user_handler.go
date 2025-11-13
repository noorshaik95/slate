package grpc

import (
	"context"

	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/internal/models"
	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/internal/service"
	pb "github.com/noorshaik95/axum-grafana-example/services/user-auth-service/api/proto"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/emptypb"
	"google.golang.org/protobuf/types/known/timestamppb"
)

type UserServiceServer struct {
	pb.UnimplementedUserServiceServer
	userService *service.UserService
}

func NewUserServiceServer(userService *service.UserService) *UserServiceServer {
	return &UserServiceServer{
		userService: userService,
	}
}

// Login authenticates a user
func (s *UserServiceServer) Login(ctx context.Context, req *pb.LoginRequest) (*pb.LoginResponse, error) {
	user, tokens, err := s.userService.Login(req.Email, req.Password)
	if err != nil {
		return nil, status.Errorf(codes.Unauthenticated, "login failed: %v", err)
	}

	return &pb.LoginResponse{
		AccessToken:  tokens.AccessToken,
		RefreshToken: tokens.RefreshToken,
		User:         userToProto(user),
		ExpiresIn:    tokens.ExpiresIn,
	}, nil
}

// Register registers a new user
func (s *UserServiceServer) Register(ctx context.Context, req *pb.RegisterRequest) (*pb.RegisterResponse, error) {
	user, tokens, err := s.userService.Register(req.Email, req.Password, req.FirstName, req.LastName, req.Phone)
	if err != nil {
		return nil, status.Errorf(codes.AlreadyExists, "registration failed: %v", err)
	}

	return &pb.RegisterResponse{
		AccessToken:  tokens.AccessToken,
		RefreshToken: tokens.RefreshToken,
		User:         userToProto(user),
	}, nil
}

// RefreshToken refreshes an access token
func (s *UserServiceServer) RefreshToken(ctx context.Context, req *pb.RefreshTokenRequest) (*pb.RefreshTokenResponse, error) {
	tokens, err := s.userService.RefreshToken(req.RefreshToken)
	if err != nil {
		return nil, status.Errorf(codes.Unauthenticated, "token refresh failed: %v", err)
	}

	return &pb.RefreshTokenResponse{
		AccessToken:  tokens.AccessToken,
		RefreshToken: tokens.RefreshToken,
		ExpiresIn:    tokens.ExpiresIn,
	}, nil
}

// ValidateToken validates a token
func (s *UserServiceServer) ValidateToken(ctx context.Context, req *pb.ValidateTokenRequest) (*pb.ValidateTokenResponse, error) {
	userID, roles, err := s.userService.ValidateToken(req.Token)
	if err != nil {
		return &pb.ValidateTokenResponse{
			Valid: false,
			Error: err.Error(),
		}, nil
	}

	return &pb.ValidateTokenResponse{
		Valid:  true,
		UserId: userID,
		Roles:  roles,
	}, nil
}

// Logout logs out a user (placeholder - implement token blacklist if needed)
func (s *UserServiceServer) Logout(ctx context.Context, req *pb.LogoutRequest) (*emptypb.Empty, error) {
	// TODO: Implement token blacklist or revocation
	return &emptypb.Empty{}, nil
}

// CreateUser creates a new user
func (s *UserServiceServer) CreateUser(ctx context.Context, req *pb.CreateUserRequest) (*pb.UserResponse, error) {
	user, err := s.userService.CreateUser(req.Email, req.Password, req.FirstName, req.LastName, req.Phone, req.Roles)
	if err != nil {
		return nil, status.Errorf(codes.AlreadyExists, "failed to create user: %v", err)
	}

	return &pb.UserResponse{
		User: userToProto(user),
	}, nil
}

// GetUser retrieves a user by ID
func (s *UserServiceServer) GetUser(ctx context.Context, req *pb.GetUserRequest) (*pb.UserResponse, error) {
	user, err := s.userService.GetUser(req.UserId)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "user not found: %v", err)
	}

	return &pb.UserResponse{
		User: userToProto(user),
	}, nil
}

// UpdateUser updates a user
func (s *UserServiceServer) UpdateUser(ctx context.Context, req *pb.UpdateUserRequest) (*pb.UserResponse, error) {
	var email, firstName, lastName, phone *string
	var isActive *bool

	if req.Email != nil {
		email = req.Email
	}
	if req.FirstName != nil {
		firstName = req.FirstName
	}
	if req.LastName != nil {
		lastName = req.LastName
	}
	if req.Phone != nil {
		phone = req.Phone
	}
	if req.IsActive != nil {
		isActive = req.IsActive
	}

	user, err := s.userService.UpdateUser(req.UserId, email, firstName, lastName, phone, isActive)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to update user: %v", err)
	}

	return &pb.UserResponse{
		User: userToProto(user),
	}, nil
}

// DeleteUser deletes a user
func (s *UserServiceServer) DeleteUser(ctx context.Context, req *pb.DeleteUserRequest) (*emptypb.Empty, error) {
	if err := s.userService.DeleteUser(req.UserId); err != nil {
		return nil, status.Errorf(codes.Internal, "failed to delete user: %v", err)
	}

	return &emptypb.Empty{}, nil
}

// ListUsers retrieves a paginated list of users
func (s *UserServiceServer) ListUsers(ctx context.Context, req *pb.ListUsersRequest) (*pb.ListUsersResponse, error) {
	var isActive *bool
	if req.IsActive != nil {
		isActive = req.IsActive
	}

	role := ""
	if req.Role != nil {
		role = *req.Role
	}

	users, total, err := s.userService.ListUsers(int(req.Page), int(req.PageSize), req.Search, role, isActive)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to list users: %v", err)
	}

	pbUsers := make([]*pb.User, len(users))
	for i, user := range users {
		pbUsers[i] = userToProto(user)
	}

	return &pb.ListUsersResponse{
		Users:    pbUsers,
		Total:    int32(total),
		Page:     req.Page,
		PageSize: req.PageSize,
	}, nil
}

// GetProfile retrieves a user's profile
func (s *UserServiceServer) GetProfile(ctx context.Context, req *pb.GetProfileRequest) (*pb.ProfileResponse, error) {
	profile, err := s.userService.GetProfile(req.UserId)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "profile not found: %v", err)
	}

	return &pb.ProfileResponse{
		Profile: &pb.Profile{
			UserId:    profile.UserID,
			FirstName: profile.FirstName,
			LastName:  profile.LastName,
			Email:     profile.Email,
			Phone:     profile.Phone,
			AvatarUrl: profile.AvatarURL,
			Bio:       profile.Bio,
			Roles:     profile.Roles,
			CreatedAt: timestamppb.New(profile.CreatedAt),
			UpdatedAt: timestamppb.New(profile.UpdatedAt),
		},
	}, nil
}

// UpdateProfile updates a user's profile
func (s *UserServiceServer) UpdateProfile(ctx context.Context, req *pb.UpdateProfileRequest) (*pb.ProfileResponse, error) {
	var firstName, lastName, phone, avatarURL, bio *string

	if req.FirstName != nil {
		firstName = req.FirstName
	}
	if req.LastName != nil {
		lastName = req.LastName
	}
	if req.Phone != nil {
		phone = req.Phone
	}
	if req.AvatarUrl != nil {
		avatarURL = req.AvatarUrl
	}
	if req.Bio != nil {
		bio = req.Bio
	}

	profile, err := s.userService.UpdateProfile(req.UserId, firstName, lastName, phone, avatarURL, bio)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to update profile: %v", err)
	}

	return &pb.ProfileResponse{
		Profile: &pb.Profile{
			UserId:    profile.UserID,
			FirstName: profile.FirstName,
			LastName:  profile.LastName,
			Email:     profile.Email,
			Phone:     profile.Phone,
			AvatarUrl: profile.AvatarURL,
			Bio:       profile.Bio,
			Roles:     profile.Roles,
			CreatedAt: timestamppb.New(profile.CreatedAt),
			UpdatedAt: timestamppb.New(profile.UpdatedAt),
		},
	}, nil
}

// ChangePassword changes a user's password
func (s *UserServiceServer) ChangePassword(ctx context.Context, req *pb.ChangePasswordRequest) (*emptypb.Empty, error) {
	if err := s.userService.ChangePassword(req.UserId, req.OldPassword, req.NewPassword); err != nil {
		return nil, status.Errorf(codes.InvalidArgument, "failed to change password: %v", err)
	}

	return &emptypb.Empty{}, nil
}

// AssignRole assigns a role to a user
func (s *UserServiceServer) AssignRole(ctx context.Context, req *pb.AssignRoleRequest) (*emptypb.Empty, error) {
	if err := s.userService.AssignRole(req.UserId, req.Role); err != nil {
		return nil, status.Errorf(codes.Internal, "failed to assign role: %v", err)
	}

	return &emptypb.Empty{}, nil
}

// RemoveRole removes a role from a user
func (s *UserServiceServer) RemoveRole(ctx context.Context, req *pb.RemoveRoleRequest) (*emptypb.Empty, error) {
	if err := s.userService.RemoveRole(req.UserId, req.Role); err != nil {
		return nil, status.Errorf(codes.Internal, "failed to remove role: %v", err)
	}

	return &emptypb.Empty{}, nil
}

// GetUserRoles retrieves all roles for a user
func (s *UserServiceServer) GetUserRoles(ctx context.Context, req *pb.GetUserRolesRequest) (*pb.GetUserRolesResponse, error) {
	roles, err := s.userService.GetUserRoles(req.UserId)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to get user roles: %v", err)
	}

	return &pb.GetUserRolesResponse{
		Roles: roles,
	}, nil
}

// CheckPermission checks if a user has a specific permission
func (s *UserServiceServer) CheckPermission(ctx context.Context, req *pb.CheckPermissionRequest) (*pb.CheckPermissionResponse, error) {
	hasPermission, err := s.userService.CheckPermission(req.UserId, req.Permission)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to check permission: %v", err)
	}

	return &pb.CheckPermissionResponse{
		HasPermission: hasPermission,
	}, nil
}

// Helper function to convert domain user to proto user
func userToProto(user *models.User) *pb.User {
	return &pb.User{
		Id:        user.ID,
		Email:     user.Email,
		FirstName: user.FirstName,
		LastName:  user.LastName,
		Phone:     user.Phone,
		Roles:     user.Roles,
		IsActive:  user.IsActive,
		CreatedAt: timestamppb.New(user.CreatedAt),
		UpdatedAt: timestamppb.New(user.UpdatedAt),
	}
}
